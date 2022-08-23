#![allow(dead_code)]
use crate::midi::sysex::{is_response_to_message, message_answer_code};

use super::{
  commands::Command,
  constants::ResponseStatusCode,
  device::{LumatoneDevice, LumatoneIO},
  error::LumatoneMidiError,
  sysex::EncodedSysex,
};
use std::{pin::Pin, time::Duration, collections::VecDeque};

use futures::Future;
use log::{debug, error, info, warn};
use tokio::{
  sync::mpsc,
  time::{sleep, Sleep},
};

use error_stack::{Result, IntoReport, ResultExt};

// state machine design is based around this example: https://play.rust-lang.org/?gist=ee3e4df093c136ced7b394dc7ffb78e1&version=stable&backtrace=0
// linked from "Pretty State Machine Patterns in Rust": https://hoverbear.org/blog/rust-state-machine-pattern/

/// One of the possible states the MIDI driver can be in at any given time.
#[derive(Debug)]
enum State {
  /// We have nothing to send, and are not waiting for anything specific to happen.
  Idle,

  /// We have one or more MIDI messages queued up to send.
  ProcessingQueue { send_queue: VecDeque<Command> },

  /// We've sent a message to the device and are waiting for a response.
  /// We may also have messages queued up to send later.
  AwaitingResponse {
    send_queue: VecDeque<Command>,
    command_sent: Command,
  },

  /// We've sent a message to the device, but the device says it's busy,
  /// so we're hanging onto the outgoing message to try again in a bit.
  /// We may also have messages queued up to send later.
  DeviceBusy {
    send_queue: VecDeque<Command>,
    to_retry: Command,
  },

  /// Something has gone horribly wrong, and we've shut down the state machine loop.
  Failed(LumatoneMidiError),
}

/// Actions are inputs into the state machine.
/// An Action may trigger a state transition, but not all actions are applicable to all states.
/// See the code of [`State::next`] for the valid (action, state) pairings.
#[derive(Debug)]
enum Action {
  /// A user of the driver has submitted a command to send to the device.
  SubmitCommand(Command),

  /// The driver has sent a command on the MIDI out port.
  MessageSent(Command),

  /// The driver has received a message on the MIDI in port.
  MessageReceived(EncodedSysex),

  /// The receive timeout has tripped while waiting for a response.
  ResponseTimedOut,

  /// The retry timeout has tripped while waiting to retry a message send.
  ReadyToRetry,
}

/// Effects are requests from the state machine to "do something" in the outside world.
#[derive(Debug)]
enum Effect {
  /// The state machine has a message ready to send on the MIDI out port.
  SendMidiMessage(Command),

  /// The state machine wants to start the receive timeout.
  StartReceiveTimeout,

  /// The state machine wants to start the busy/retry timeout.
  StartRetryTimeout,
}

impl State {
  /// Applies an Action to the current State and returns the new State.
  /// Note that this may be the same as the original state, in cases where the given
  /// Action does not apply to the current state.
  fn next(self, action: Action) -> State {
    use Action::*;
    use State::*;

    debug!("handling action {:?}. current state: {:?}", action, self);
    match (action, self) {
      // Submitting a command in the Idle state transitions to ProcessingQueue, with the new message as the only queue member.
      (SubmitCommand(cmd), Idle) => {
        let mut send_queue = VecDeque::new();
        send_queue.push_back(cmd);
        ProcessingQueue {
          send_queue, 
        }
      },

      // Submitting a command while we're waiting for a response to a previous command transitions to a new
      // AwaitingResponse state with the new command pushed onto the send queue.
      (
        SubmitCommand(cmd),
        AwaitingResponse {
          send_queue,
          command_sent,
        },
      ) => {
        // add new command to the send_queue
        let mut q = send_queue;
        q.push_back(cmd);
        AwaitingResponse {
          send_queue: q,
          command_sent: command_sent,
        }
      }

      // Submitting a commmand while we're waiting to retry a previous command transitions to a new
      // DeviceBusy state with the new command pushed onto the send queue.
      (
        SubmitCommand(cmd),
        DeviceBusy {
          send_queue,
          to_retry,
        },
      ) => {
        // add new command to the send queue
        let mut q = send_queue;
        q.push_back(cmd);
        DeviceBusy {
          send_queue: q,
          to_retry: to_retry,
        }
      }

      // Submitting a command while we're processing the queue transitions to a new ProcessingQueue state
      // with the new command pushed onto the queue.
      (SubmitCommand(cmd), ProcessingQueue { send_queue }) => {
        let mut q = send_queue;
        q.push_back(cmd);
        ProcessingQueue { send_queue: q }
      }

      // Getting confirmation that a message was sent out while we're processing the queue transitions to
      // the AwaitingResponse state.
      (MessageSent(msg), ProcessingQueue { send_queue }) => {
        let mut q = send_queue;
        q.pop_front();
        AwaitingResponse {
          send_queue: q,
          command_sent: msg,
        }
      }

      // Receiving a message when we're awaiting a response transitions to one of several states, depending on
      // the response data:
      // - If the response indicates that the device is busy or in demo mode, transitions to the DeviceBusy state.
      // - If the response is not a valid response for the message we sent, logs a warning message drops the response.
      // - If the response is valid, transitions to ProcessingQueue or Idle (if queue is empty).
      (
        MessageReceived(incoming),
        AwaitingResponse {
          send_queue,
          command_sent: outgoing,
        },
      ) => {
        use ResponseStatusCode::{Busy, State};
        if !is_response_to_message(&outgoing.to_sysex_message(), &incoming) {
          warn!("received message that doesn't match expected response. outgoing message: {:?} - incoming: {:?}", outgoing, incoming);
        }

        let status = message_answer_code(&incoming);
        log_message_status(&status, &outgoing);

        match (status, send_queue.is_empty()) {
          (Busy, _) => DeviceBusy {
            send_queue: send_queue,
            to_retry: outgoing,
          },

          (State, _) => DeviceBusy {
            send_queue: send_queue,
            to_retry: outgoing,
          },

          (_, true) => Idle,

          (_, false) => ProcessingQueue { send_queue },
        }
      }

      // Receiving a message when we're not expecting one logs a warning.
      (MessageReceived(msg), state) => {
        warn!("Message received when not awaiting response: {:?}", msg);
        state
      }

      // Getting a ResponseTimedOut action while waiting for a response logs a warning
      // and transitions to Idle or ProcessingQueue, depending on whether we have messages queued up.
      (
        ResponseTimedOut,
        AwaitingResponse {
          send_queue,
          command_sent,
        },
      ) => {
        warn!("Timed out waiting for response to msg: {:?}", command_sent);

        if send_queue.is_empty() {
          Idle
        } else {
          ProcessingQueue {
            send_queue: send_queue,
          }
        }
      }

      // Getting a ResponseTimedOut when we're not waiting for a response logs a warning.
      (ResponseTimedOut, state) => {
        warn!("Response timeout action received, but not awaiting response");
        state
      }

      // Getting a ReadyToRetry action when we're in the DeviceBusy state transitions to ProcessingQueue,
      // with the message to retry added to the front of the queue (so it will be sent next).
      (
        ReadyToRetry,
        DeviceBusy {
          send_queue,
          to_retry,
        },
      ) => {
        let mut q = send_queue;
        q.push_front(to_retry);
        ProcessingQueue { send_queue: q }
      }

      // Getting a ReadyToRetry action in any state except DeviceBusy logs a warning.
      (ReadyToRetry, state) => {
        warn!("ReadyToRetry action received but not in DeviceBusy state");
        state
      }

      // All other state transitions are undefined and result in a Failed state, causing the driver loop to exit with an error.
      (action, state) => {
        let msg = format!("invalid action {:?} for current state {:?}", action, state);
        Failed(LumatoneMidiError::InvalidStateTransition(msg))
      }
    }
  }

  /// Each state can perform an optional Effect when it's entered.
  /// Effects may result in new Actions, which are fed into `State::next` and can then trigger a new State transition.
  fn enter(&mut self) -> Option<Effect> {
    use Effect::*;
    use State::*;

    debug!("entering state {:?}", self);

    match self {
      Idle => None,
      ProcessingQueue { send_queue } => {
        send_queue.pop_front().map(|cmd| SendMidiMessage(cmd))
      }
      DeviceBusy {
        send_queue: _,
        to_retry: _,
      } => Some(StartRetryTimeout),
      AwaitingResponse {
        send_queue: _,
        command_sent: _,
      } => Some(StartReceiveTimeout),
      Failed(err) => {
        warn!("midi driver - unrecoverable error: {err}");
        None
      }
    }
  }
}

/// The MidiDriver controls the MIDI I/O event loop / state machine.
struct MidiDriverInternal {
  device_io: LumatoneIO,
  receive_timeout: Option<Pin<Box<Sleep>>>,
  retry_timeout: Option<Pin<Box<Sleep>>>,
}


pub struct MidiDriver {
  command_tx: mpsc::Sender<Command>,
  done_tx: mpsc::Sender<()>,
}

impl MidiDriver {
  pub async fn send(&self, cmd: Command) -> Result<(), LumatoneMidiError> {
    self.command_tx.send(cmd).await
      .report()
      .change_context(LumatoneMidiError::DeviceSendError)
  }

  pub fn blocking_send(&self, cmd: Command) -> Result<(), LumatoneMidiError> {
    self.command_tx.blocking_send(cmd)
      .report()
      .change_context(LumatoneMidiError::DeviceSendError)
  }

  pub async fn done(&self) -> Result<(), LumatoneMidiError> {
    self.done_tx.send(()).await
      .report()
      .change_context(LumatoneMidiError::DeviceSendError)
  }
}


impl MidiDriver {
  pub fn new(device: &LumatoneDevice) -> Result<(MidiDriver, impl Future<Output = ()>), LumatoneMidiError> {
    let internal = MidiDriverInternal::new(device)?;
    let (command_tx, command_rx) = mpsc::channel(128);
    let (done_tx, done_rx) = mpsc::channel(1);
    
    let driver = MidiDriver { command_tx, done_tx };
    Ok((driver, internal.run(command_rx, done_rx)))
  }
}


impl MidiDriverInternal {
  fn new(device: &LumatoneDevice) -> Result<Self, LumatoneMidiError> {
    let device_io = device.connect()?;
    Ok(MidiDriverInternal {
      device_io,
      receive_timeout: None,
      retry_timeout: None,
    })
  }

  /// Performs some Effect. On success, returns an Option<Action> to potentially trigger a state transition.
  fn perform_effect(&mut self, effect: Effect) -> Result<Option<Action>, LumatoneMidiError> {
    use Action::*;
    use Effect::*;
    let action = match effect {
      SendMidiMessage(cmd) => {
        self.device_io.send(&cmd.to_sysex_message())?;
        Some(MessageSent(cmd))
      }

      StartReceiveTimeout => {
        let timeout_sec = 30;
        let timeout = sleep(Duration::from_secs(timeout_sec));
        self.receive_timeout = Some(Box::pin(timeout));
        None
      }

      StartRetryTimeout => {
        let timeout_sec = 3;
        let timeout = sleep(Duration::from_secs(timeout_sec));
        self.retry_timeout = Some(Box::pin(timeout));
        None
      }
    };
    Ok(action)
  }

  /// Run the MidiDriver I/O event loop.
  /// Commands to send to the device should be sent on the `commands` channel.
  ///
  /// To exit the loop, send `()` on the `done_signal` channel.
  ///
  /// TODO: rethink this API (passing in channels is a bit awkward)
  ///
  /// TODO: add some kind of feedback mechanism for failed message sends, etc
  ///       e.g. the "listener" interface from the typescript version.
  ///       Alternatively, a "command" could be an EncodedSysex + a oneshot channel to report status back.
  async fn run(
    mut self,
    mut commands: mpsc::Receiver<Command>,
    mut done_signal: mpsc::Receiver<()>,
  ) {
    let mut state = State::Idle;
    loop {
      // bail out if instructed
      if done_signal.try_recv().is_ok() {
        debug!("done signal received, exiting");
        break;
      }

      // if either timeout is None, use a timeout with Duration::MAX, to make the select! logic a bit simpler
      let mut receive_timeout = &mut Box::pin(sleep(Duration::MAX));
      if let Some(t) = &mut self.receive_timeout {
        receive_timeout = t;
      }

      let mut retry_timeout = &mut Box::pin(sleep(Duration::MAX));
      if let Some(t) = &mut self.retry_timeout {
        retry_timeout = t;
      }

      // There are two incoming streams of information: incoming midi messages,
      // and incoming commands (requests to send out midi messages)
      // There are also two timeouts: receive_timeout for when we're waiting for a response to a command,
      // and retry_timeout for when we're waiting to re-send a command (because the device was busy last time).
      //
      // This select pulls whatever is available next and maps it to an Action that will advance the state machine.
      let a = tokio::select! {
        _ = receive_timeout => {
          info!("receive timeout triggered");
          Action::ResponseTimedOut
        },

        _ = retry_timeout => {
          info!("retry timeout triggered");
          Action::ReadyToRetry
        },

        Some(msg) = self.device_io.incoming_messages.recv() => {
          info!("message received, forwarding to state machine");
          Action::MessageReceived(msg)
        }

        Some(cmd) = commands.recv() => {
          Action::SubmitCommand(cmd)
        }
      };

      // Transition to next state based on action
      state = state.next(a);

      if let State::Failed(err) = state {
        // return Err(err);
        error!("state machine error: {err}");
        break;
      }

      // The new state's `enter` fn may return an Effect.
      // If so, run it and apply any Actions returned.
      if let Some(effect) = state.enter() {
        match self.perform_effect(effect) {
          Ok(Some(action)) => {
            state = state.next(action);
            if let State::Failed(err) = state {
              error!("state machine error: {err}");
              break;
            }
          }
          Err(err) => {
            // warn!("error performing effect: {}", err);
            error!("state machine error: {err}");
            break;
          }
          _ => {
            // No error, but nothing to do
          }
        }
      }
    }

    // Ok(())
  }
}

fn log_message_status(status: &ResponseStatusCode, outgoing: &Command) {
  use ResponseStatusCode::*;
  match *status {
    Nack => debug!("received NACK response to command {:?}", outgoing),
    Ack => {}
    Busy => debug!("received Busy response to command {:?}", outgoing),
    Error => debug!("received Error response to command {:?}", outgoing),
    State => debug!("received State response to command {:?}", outgoing),
    Unknown => warn!(
      "received unknown response status in response to command: {:?}",
      outgoing
    ),
  }
}
