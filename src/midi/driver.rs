#![allow(dead_code)]
use crate::midi::sysex::{is_response_to_message, message_answer_code};

use super::{
  commands::Command,
  constants::ResponseStatusCode,
  device::{LumatoneDevice, LumatoneIO},
  error::LumatoneMidiError,
  sysex::EncodedSysex, responses::Response,
};
use std::{pin::Pin, time::Duration, collections::VecDeque, fmt::{Display, Debug}};

use futures::{Future, TryFutureExt};
use log::{debug, error, info, warn};
use tokio::{
  sync::mpsc,
  time::{sleep, Sleep},
};

use error_stack::{Result, IntoReport, ResultExt, report, Report};

// state machine design is based around this example: https://play.rust-lang.org/?gist=ee3e4df093c136ced7b394dc7ffb78e1&version=stable&backtrace=0
// linked from "Pretty State Machine Patterns in Rust": https://hoverbear.org/blog/rust-state-machine-pattern/
// with the addition of an explicit `Effect` type to model side effects

type ResponseResult = Result<Response, LumatoneMidiError>;

#[derive(Clone)]
struct CommandSubmission {
  command: Command, 
  response_tx: mpsc::Sender<ResponseResult>,
}

impl Debug for CommandSubmission {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    f.debug_struct("CommandSubmission")
      .field("command", &self.command)
      .field("response_tx", &"(opaque)")
      .finish()
  }
}

impl Display for CommandSubmission {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    write!(f, "CommandSubmission({})", self.command)
  }
}

/// One of the possible states the MIDI driver can be in at any given time.
#[derive(Debug)]
enum State {
  /// We have nothing to send, and are not waiting for anything specific to happen.
  Idle,

  /// We have one or more MIDI messages queued up to send.
  ProcessingQueue { send_queue: VecDeque<CommandSubmission> },

  /// We've sent a message to the device and are waiting for a response.
  /// We may also have messages queued up to send later.
  AwaitingResponse {
    send_queue: VecDeque<CommandSubmission>,
    command_sent: CommandSubmission,
  },

  // We've unpacked a Response from a device message and are ready to
  // notify the user.
  ProcessingResponse {
    send_queue: VecDeque<CommandSubmission>,
    command_sent: CommandSubmission,
    response_msg: EncodedSysex, 
  },

  /// We've sent a message to the device, but the device says it's busy,
  /// so we're hanging onto the outgoing message to try again in a bit.
  /// We may also have messages queued up to send later.
  DeviceBusy {
    send_queue: VecDeque<CommandSubmission>,
    to_retry: CommandSubmission,
  },

  /// Something has gone horribly wrong, and we've shut down the state machine loop.
  Failed(Report<LumatoneMidiError>),
}

/// Actions are inputs into the state machine.
/// An Action may trigger a state transition, but not all actions are applicable to all states.
/// See the code of [`State::next`] for the valid (action, state) pairings.
#[derive(Debug)]
enum Action {
  /// A user of the driver has submitted a command to send to the device.
  SubmitCommand(CommandSubmission),

  /// The driver has sent a command on the MIDI out port.
  MessageSent(CommandSubmission),

  /// The driver has received a message on the MIDI in port.
  MessageReceived(EncodedSysex),

  /// We've informed users about a command response and are ready to
  ///  advance out of the ProcessingResponse state.
  ResponseDispatched,

  /// The receive timeout has tripped while waiting for a response.
  ResponseTimedOut,

  /// The retry timeout has tripped while waiting to retry a message send.
  ReadyToRetry,
}

/// Effects are requests from the state machine to "do something" in the outside world.
#[derive(Debug)]
enum Effect {
  /// The state machine has a message ready to send on the MIDI out port.
  SendMidiMessage(CommandSubmission),

  /// The state machine wants to start the receive timeout.
  StartReceiveTimeout,

  /// The state machine wants to start the busy/retry timeout.
  StartRetryTimeout,

  /// The state machine has received a response to a message and wants to notify
  /// the outside world about its success or failure.
  NotifyMessageResponse(CommandSubmission, Result<Response, LumatoneMidiError>),
}

impl State {
  /// Applies an Action to the current State and returns the new State.
  /// Note that this may be the same as the original state, in cases where the given
  /// Action does not apply to the current state.
  fn next(self, action: Action) -> State {
    use Action::*;
    use State::*;

    // debug!("handling action {:?}. current state: {:?}", action, self);
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
        AwaitingResponse {
          send_queue,
          command_sent: msg,
        }
      }

      // Receiving a message when we're awaiting a response transitions to ProcessingResponse
      (
        MessageReceived(response_msg),
        AwaitingResponse {
          send_queue,
          command_sent,
        },
      ) => {
        ProcessingResponse { send_queue, command_sent, response_msg }
      }

      // Getting confirmation that we're done processing a response while we're in the ResponseProcessing state
      // transitions to either Idle or ProcessingQueue, depending on whether there are messages left to send
      (
        ResponseDispatched,
        ProcessingResponse {
          send_queue,
          ..
        }
      ) => {
        if send_queue.is_empty() {
          Idle
        } else {
          ProcessingQueue { send_queue }
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
        Failed(report!(LumatoneMidiError::InvalidStateTransition(msg)))
      }
    }
  }

  /// Each state can perform an optional Effect when it's entered, and may trigger an optional Action to feed into the state machine next.
  fn enter(&mut self) -> (Option<Effect>, Option<Action>) {
    use Effect::*;
    use State::*;

    // debug!("entering state {:?}", self);

    match self {
      Idle => (None, None),
      ProcessingQueue { send_queue } => {
        match send_queue.pop_front() {
          None => (None, None),
          Some(cmd) => (Some(SendMidiMessage(cmd.clone())), Some(Action::MessageSent(cmd))),
        }
      }
      DeviceBusy { .. } => (Some(StartRetryTimeout), None),
      AwaitingResponse { .. } => (Some(StartReceiveTimeout), None),
      ProcessingResponse { command_sent, response_msg, .. } => {
        if !is_response_to_message(&command_sent.command.to_sysex_message(), &response_msg) {
          warn!("received message that doesn't match expected response. outgoing message: {:?} - incoming: {:?}", command_sent.command, response_msg);
        }

        let status = message_answer_code(&response_msg);
        log_message_status(&status, &command_sent.command);

        // TODO: check status for Busy / State and dispatch actions to enter the "waiting to retry" state

        let response_res = Response::from_sysex_message(response_msg);

        let effect = NotifyMessageResponse(command_sent.clone(), response_res);
        (Some(effect), Some(Action::ResponseDispatched))
      }
      Failed(err) => {
        error!("midi driver - unrecoverable error: {err}");
        (None, None) // todo: return ExitWithError effect
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
  command_tx: mpsc::Sender<CommandSubmission>,
  done_tx: mpsc::Sender<()>,
}

impl MidiDriver {
  pub async fn send(&self, command: Command) -> Result<Response, LumatoneMidiError> {
    let (response_tx, mut response_rx) = mpsc::channel(1);
    let submission = CommandSubmission { command, response_tx };
    let send_f = self.command_tx.send(submission)
      .map_err(|e| report!(e).change_context(LumatoneMidiError::DeviceSendError));

    send_f.await?;
    response_rx.recv().await.unwrap()
  }

  pub fn blocking_send(&self, command: Command) -> Result<mpsc::Receiver<ResponseResult>, LumatoneMidiError> {
    let (response_tx, response_rx) = mpsc::channel(1);
    let submission = CommandSubmission { command, response_tx };
    self.command_tx.blocking_send(submission)
      .report()
      .change_context(LumatoneMidiError::DeviceSendError)?;
    Ok(response_rx)
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

  /// Performs some Effect. 
  async fn perform_effect(&mut self, effect: Effect) -> Result<(), LumatoneMidiError> {
    use Effect::*;
    match effect {
      SendMidiMessage(cmd) => {
        self.device_io.send(&cmd.command.to_sysex_message())?;
      }
      StartReceiveTimeout => {
        let timeout_sec = 30;
        let timeout = sleep(Duration::from_secs(timeout_sec));
        self.receive_timeout = Some(Box::pin(timeout));
      }
      StartRetryTimeout => {
        let timeout_sec = 3;
        let timeout = sleep(Duration::from_secs(timeout_sec));
        self.retry_timeout = Some(Box::pin(timeout));
      },
      NotifyMessageResponse(cmd_submission, result) => {
        if let Err(err) = cmd_submission.response_tx.send(result).await {
          error!("error sending response notification: {err}");
        }
      }
    };
    Ok(())
  }

  /// Run the MidiDriver I/O event loop.
  /// Commands to send to the device should be sent on the `commands` channel.
  ///
  /// To exit the loop, send `()` on the `done_signal` channel.
  ///
  async fn run(
    mut self,
    mut commands: mpsc::Receiver<CommandSubmission>,
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
          self.receive_timeout = None;
          Action::ResponseTimedOut
        },

        _ = retry_timeout => {
          info!("retry timeout triggered");
          self.retry_timeout = None;
          Action::ReadyToRetry
        },

        Some(msg) = self.device_io.incoming_messages.recv() => {
          info!("message received, forwarding to state machine");
          self.receive_timeout = None;
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

      // The new state's `enter` fn may return an Effect and/or an Action.
      // If there's an effect, perform it. If there's an action, feed it into state.next()
      // to advance the state machine.
      let (maybe_effect, maybe_action) = state.enter();
      if let Some(effect) = maybe_effect {
        if let Err(err) = self.perform_effect(effect).await {
          state = State::Failed(err);
        }
      }
      if let Some(action) = maybe_action {
        state = state.next(action);
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
