#![allow(dead_code)]

//! Implements a driver for the Lumatone's Midi SysEx protocol using a finite state machine.
//!
//! ## Public API
//!
//! The [MidiDriver] provides a [`send`](MidiDriver::send) method that will queue up a [Command]
//! to send to the device. `send` is an async method whose Future will resolve when the device
//! returns a [Response] or an error occurs.
//!
//! To create a [MidiDriver], use [MidiDriver::new], which returns a tuple of
//! `(MidiDriver, Future)`. The Future needs to be spawned and `await`ed in order to start the
//! driver's event loop.
//!
//! To shutdown the driver loop, use [MidiDriver::done].
//!
//!
//! ## State machine internals
//!
//! State machine design is based around [this example](https://play.rust-lang.org/?gist=ee3e4df093c136ced7b394dc7ffb78e1&version=stable&backtrace=0)
//! linked from ["Pretty State Machine Patterns in Rust"](https://hoverbear.org/blog/rust-state-machine-pattern/)
//! with the addition of an explicit `Effect` type to model side effects
//!
//! Rough state machine flow:
//!
//! ```text
//!                       ┌──────┐
//!          ┌───────────►│ Idle │
//!          │            └──┬───┘
//!          │               │
//!          │               │ SubmitCommand
//!          │               │
//!          │          ┌────▼───────────────┐
//!    QueueEmpty───────┤                    │  SubmitCommand
//!     ┌──────────────►│  ProcessingQueue   ◄─────────┐
//!     │       ┌──────►│                    ┌─────────┘
//!     │       │       └────┬──────────────▲┘
//!     │       │            │              │
//!     │       │            │ MessageSent  └──────────────────────┐
//!     │       │            │                                     │
//!     │       │       ┌────▼───────────────┐                     │
//!     │       │       │                    │ SubmitCommand       │
//!     │       │       │  AwaitingResponse  ◄────────┐            │
//! ResponseTimedOut────│                    ┌────────┘            │
//!             │       └────┬───────────────┘                     │
//!             │            │                                     │
//!             │            │                                     │
//!             │            │                                     │
//!             │            │                                     │
//!             │            │ MessageReceived        ReadyToRetry │
//!             │            │                                     │
//!             │       ┌────▼─────────────────┐            ┌──────┴───────────┐
//!             │       │                      │            │                  │
//!             │       │  ProcessingResponse  │            │  WaitingToRetry  │
//! ResponseDispatched──┤                      │─DeviceBusy─►                  │
//!                     └───────┬──▲───────────┘            └───────┬──▲───────┘
//!                             │  │                                │  │
//!               SubmitCommand └──┘                  SubmitCommand └──┘
//! ```

use super::{
  commands::Command,
  constants::ResponseStatusCode,
  device::{LumatoneDevice, LumatoneIO},
  error::LumatoneMidiError,
  responses::Response,
  sysex::{is_response_to_message, message_answer_code, EncodedSysex},
};
use std::{
  collections::VecDeque,
  fmt::{Debug, Display},
  pin::Pin,
  time::Duration,
};

use futures::{Future, TryFutureExt};
use log::{debug, error, info, warn};
use tokio::{
  sync::mpsc,
  time::{sleep, Sleep},
};

use error_stack::{report, IntoReport, Report, Result, ResultExt};
use crate::driver::Action::QueueEmpty;


/// Result type returned in response to a command submission
type ResponseResult = Result<Response, LumatoneMidiError>;

/// Request to send a command to the device, with a channel to send a response on.
#[derive(Clone)]
struct CommandSubmission {
  command: Command,
  response_tx: mpsc::Sender<ResponseResult>,
}

impl CommandSubmission {
  /// Creates a new CommandSubmission and returns it, along with the receive channel
  /// for the command's [ResponseResult].
  fn new(command: Command) -> (Self, mpsc::Receiver<ResponseResult>) {
    let (response_tx, response_rx) = mpsc::channel(1);
    let sub = CommandSubmission { command, response_tx };
    (sub, response_rx)
  }
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
  ProcessingQueue {
    send_queue: VecDeque<CommandSubmission>,
  },

  /// We've sent a message to the device and are waiting for a response.
  /// We may also have messages queued up to send later.
  AwaitingResponse {
    send_queue: VecDeque<CommandSubmission>,
    command_sent: CommandSubmission,
  },

  /// We've unpacked a Response from a device message and are ready to
  /// notify the user.
  ProcessingResponse {
    send_queue: VecDeque<CommandSubmission>,
    command_sent: CommandSubmission,
    response_msg: EncodedSysex,
  },

  /// We've sent a message to the device, but the device says it's busy,
  /// so we're hanging onto the outgoing message to try again in a bit.
  /// We may also have messages queued up to send later.
  WaitingToRetry {
    send_queue: VecDeque<CommandSubmission>,
    to_retry: CommandSubmission,
  },

  /// Something has gone horribly wrong, and we've shut down the state machine loop.
  Failed(Report<LumatoneMidiError>),
}

impl Display for State {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    use State::*;
    match self {
      Idle => write!(f, "Idle"),
      ProcessingQueue { send_queue } => write!(f, "ProcessingQueue({} in queue)", send_queue.len()),
      AwaitingResponse {
        send_queue,
        command_sent,
      } => write!(
        f,
        "AwaitingResponse({}, {} in queue)",
        command_sent.command,
        send_queue.len()
      ),
      ProcessingResponse {
        send_queue,
        command_sent,
        response_msg,
      } => write!(
        f,
        "ProcessingResponse({}, {}, {} in queue)",
        command_sent.command,
        to_hex_debug_str(response_msg),
        send_queue.len()
      ),
      WaitingToRetry {
        send_queue,
        to_retry,
      } => write!(
        f,
        "WaitingToRetry({}, {} in queue)",
        to_retry.command,
        send_queue.len()
      ),
      Failed(err) => write!(f, "Failed({:?})", err),
    }
  }
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

  /// The device has signaled that it can't process the last command we sent,
  /// and we should back off for a bit before trying again.
  DeviceBusy,

  /// We've informed users about a command response and are ready to
  ///  advance out of the ProcessingResponse state.
  ResponseDispatched,

  /// The receive timeout has tripped while waiting for a response.
  ResponseTimedOut,

  /// The retry timeout has tripped while waiting to retry a message send.
  ReadyToRetry,

  /// The send queue is empty, and we can return to the Idle state.
  QueueEmpty,
}

impl Display for Action {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    use Action::*;
    match self {
      SubmitCommand(cmd) => write!(f, "SubmitCommand({})", cmd.command),
      MessageSent(cmd) => write!(f, "MessageSent({})", cmd.command),
      MessageReceived(msg) => write!(f, "MessageReceived({:?} ...)", to_hex_debug_str(msg)),
      DeviceBusy => write!(f, "DeviceBusy"),
      ResponseDispatched => write!(f, "ResponseDispatched"),
      ResponseTimedOut => write!(f, "ResponseTimedOut"),
      ReadyToRetry => write!(f, "ReadyToRetry"),
      QueueEmpty => write!(f, "QueueEmpty"),
    }
  }
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

impl Display for Effect {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    use Effect::*;
    match self {
      SendMidiMessage(cmd) => write!(f, "SendMidiMessage({})", cmd.command),
      StartReceiveTimeout => write!(f, "StartReceiveTimeout"),
      StartRetryTimeout => write!(f, "StartRetryTimeout"),
      NotifyMessageResponse(cmd, res) => {
        write!(f, "NotfiyMessageResponse({}, {:?})", cmd.command, res)
      }
    }
  }
}

impl State {
  /// Applies an [Action] to the current [State] and returns the new State.
  /// Note that this may be the same as the original state, in cases where the given
  /// Action does not apply to the current state.
  fn next(self, action: Action) -> State {
    use Action::*;
    use State::*;

    // debug!("Current state: {} --- action: {}", self, action);

    // debug!("handling action {:?}. current state: {:?}", action, self);
    match (action, self) {
      // Submitting a command in the Idle state transitions to ProcessingQueue, with the new message as the only queue member.
      (SubmitCommand(cmd), Idle) => {
        let mut send_queue = VecDeque::new();
        send_queue.push_back(cmd);
        ProcessingQueue { send_queue }
      }

      // Submitting a command while we're waiting for a response to a previous command transitions to a new
      // AwaitingResponse state with the new command pushed onto the send queue.
      (
        SubmitCommand(cmd),
        AwaitingResponse {
          mut send_queue,
          command_sent,
        },
      ) => {
        // add new command to the send_queue
        send_queue.push_back(cmd);
        AwaitingResponse {
          send_queue,
          command_sent,
        }
      }

      // Submitting a commmand while we're waiting to retry a previous command transitions to a new
      // WaitingToRetry state with the new command pushed onto the send queue.
      (
        SubmitCommand(cmd),
        WaitingToRetry {
          mut send_queue,
          to_retry,
        },
      ) => {
        // add new command to the send queue
        send_queue.push_back(cmd);
        WaitingToRetry {
          send_queue,
          to_retry,
        }
      }

      // Submitting a command while we're processing the queue transitions to a new ProcessingQueue state
      // with the new command pushed onto the queue.
      (SubmitCommand(cmd), ProcessingQueue { mut send_queue }) => {
        send_queue.push_back(cmd);
        ProcessingQueue { send_queue }
      }

      // Submitting a command while we're processing a response transitions to a new ProcessingResponse state
      // with the new command pushed onto the queue
      (SubmitCommand(cmd), ProcessingResponse { mut send_queue, command_sent, response_msg }) => {
        send_queue.push_back(cmd);
        ProcessingResponse { send_queue, command_sent, response_msg }
      }

      // Getting confirmation that a message was sent out while we're processing the queue transitions to
      // the AwaitingResponse state.
      (MessageSent(command_sent), ProcessingQueue { send_queue }) => AwaitingResponse {
        send_queue,
        command_sent,
      },

      // Receiving a message when we're awaiting a response transitions to ProcessingResponse
      (
        MessageReceived(response_msg),
        AwaitingResponse {
          send_queue,
          command_sent,
        },
      ) => ProcessingResponse {
        send_queue,
        command_sent,
        response_msg,
      },

      // Receiving a message when we're not expecting one logs a warning.
      (MessageReceived(msg), state) => {
        warn!(
          "Message received when not awaiting response. msg: {:?} current state: {}",
          to_hex_debug_str(&msg),
          state
        );
        state
      }


      // Getting confirmation that we're done processing a response while we're in the ProcessingResponse state
      // transitions to ProcessingQueue
      // TODO: add a response_msg field to ResponseDispatched action, so we can make sure it matches the one
      // in the ProcessingResponse state.
      (ResponseDispatched, ProcessingResponse { send_queue, .. }) => {
        ProcessingQueue { send_queue }
      }

      // Getting a ResponseTimedOut action while waiting for a response logs a warning
      // and transitions to ProcessingQueue.
      // TODO: this should retry or return a failure on the response channel instead of ignoring
      (
        ResponseTimedOut,
        AwaitingResponse {
          send_queue,
          command_sent,
        },
      ) => {
        warn!("Timed out waiting for response to msg: {:?}", command_sent);
        ProcessingQueue {
          send_queue,
        }
      }

      // Getting a ResponseTimedOut when we're not waiting for a response logs a warning.
      (ResponseTimedOut, state) => {
        warn!("Response timeout action received, but not awaiting response");
        state
      }

      // Getting a ReadyToRetry action when we're in the WaitingToRetry state transitions to ProcessingQueue,
      // with the message to retry added to the front of the queue (so it will be sent next).
      (
        ReadyToRetry,
        WaitingToRetry {
          mut send_queue,
          to_retry,
        },
      ) => {
        send_queue.push_front(to_retry);
        ProcessingQueue { send_queue }
      }

      // Getting a QueueEmpty action when we're in the ProcessingQueue state transitions to Idle.
      // If the queue is not actually empty, transitions to Failed, as that shouldn't happen
      (
        QueueEmpty,
        ProcessingQueue { send_queue }
      ) => {
        if !send_queue.is_empty() {
          let msg = format!("Received QueueEmpty action, but queue has {} elements", send_queue.len());
          Failed(report!(LumatoneMidiError::InvalidStateTransition(msg)))
        } else {
          Idle
        }
      }

      // Getting a ReadyToRetry action in any state except WaitingToRetry logs a warning.
      (ReadyToRetry, state) => {
        warn!("ReadyToRetry action received but not in WaitingToRetry state");
        state
      }

      // All other state transitions are undefined and result in a Failed state, causing the driver loop to exit with an error.
      (action, state) => {
        let msg = format!("invalid action {:?} for current state {:?}", action, state);
        Failed(report!(LumatoneMidiError::InvalidStateTransition(msg)))
      }
    }
  }

  /// Each state can perform an optional [Effect] when it's entered, and may trigger an optional
  /// [Action] to feed into the state machine next.
  ///
  /// Note that `enter` does not perform any effects or apply actions, just returns instructions
  /// to do so. See [MidiDriverInternal] for the bit that performs effects and advances the state
  /// machine.
  fn enter(&mut self) -> (Option<Effect>, Option<Action>) {
    use Effect::*;
    use State::*;

    // debug!("entering state {:?}", self);

    match self {
      Idle => (None, None),
      ProcessingQueue { send_queue } => match send_queue.pop_front() {
        None => (None, Some(QueueEmpty)),
        Some(cmd) => (
          Some(SendMidiMessage(cmd.clone())),
          Some(Action::MessageSent(cmd)),
        ),
      },
      WaitingToRetry { .. } => (Some(StartRetryTimeout), None),
      AwaitingResponse { .. } => (Some(StartReceiveTimeout), None),
      ProcessingResponse {
        command_sent,
        response_msg,
        ..
      } => {
        if !is_response_to_message(&command_sent.command.to_sysex_message(), &response_msg) {
          warn!("received message that doesn't match expected response. outgoing message: {} - incoming: {}", command_sent.command, to_hex_debug_str(response_msg));
        }

        let status = message_answer_code(&response_msg);
        log_message_status(&status, &command_sent.command);

        match status {
          ResponseStatusCode::Busy => {
            (None, Some(Action::DeviceBusy))
          },

          ResponseStatusCode::State => {
            warn!("device is in demo mode!");
            // FIXME: demo mode should probably have its own action that triggers
            // sending a command to exit demo mode.
            (None, Some(Action::DeviceBusy))
          },

          ResponseStatusCode::Error => {
            let res = Err(report!(LumatoneMidiError::InvalidResponseMessage("Device response had error flag set".to_string())));
            let effect = NotifyMessageResponse(command_sent.clone(), res);
            (Some(effect), Some(Action::ResponseDispatched))
          },

          ResponseStatusCode::Nack => {
            let res = Err(report!(LumatoneMidiError::InvalidResponseMessage(format!("Device sent NACK in response to command {command_sent:?}"))));
            let effect = NotifyMessageResponse(command_sent.clone(), res);
            (Some(effect), Some(Action::ResponseDispatched))
          },

          ResponseStatusCode::Ack => {
            let response_res = Response::from_sysex_message(response_msg);

            let effect = NotifyMessageResponse(command_sent.clone(), response_res);
            (Some(effect), Some(Action::ResponseDispatched))
          },

          ResponseStatusCode::Unknown => {
            // Unknown means the device sent a status code we don't know about.
            // log a warning and pretend it's all good
            warn!("Unknown response status code received");
            (None, Some(Action::ResponseDispatched))
          }
        }

      }
      Failed(err) => {
        error!("midi driver - unrecoverable error: {err}");
        (None, None) // todo: return ExitWithError effect
      }
    }
  }
}

/// An internal helper struct for the [MidiDriver] that owns the connection to the device
/// and timeouts needed by some "waiting" states.
struct MidiDriverInternal {
  device_io: LumatoneIO,
  receive_timeout: Option<Pin<Box<Sleep>>>,
  retry_timeout: Option<Pin<Box<Sleep>>>,
}

/// The MidiDriver provides an interface for sending [Command]s to a Lumatone device
/// and receiving [Response]s (or [LumatoneMidiError]s).
///
/// Use the async [send] method
pub struct MidiDriver {
  command_tx: mpsc::Sender<CommandSubmission>,
  done_tx: mpsc::Sender<()>,
}

impl MidiDriver {

  /// Sends a [Command] to the device asynchronously, returning a Future that will resolve
  /// with the Command's [Response] on success, or a [LumatoneMidiError] report on failure.
  pub async fn send(&self, command: Command) -> Result<Response, LumatoneMidiError> {
    let (submission, mut response_rx) = CommandSubmission::new(command);
    let send_f = self
      .command_tx
      .send(submission)
      .map_err(|e| report!(e).change_context(LumatoneMidiError::DeviceSendError));

    send_f.await?;
    response_rx.recv().await.unwrap()
  }

  /// Like [MidiDriver::send], but blocks the thread and returns a Result when the response is received.
  /// Must be called from a different thread than the one running the driver loop future.
  pub fn blocking_send(
    &self,
    command: Command,
  ) -> Result<mpsc::Receiver<ResponseResult>, LumatoneMidiError> {
    let (response_tx, response_rx) = mpsc::channel(1);
    let submission = CommandSubmission {
      command,
      response_tx,
    };
    self
      .command_tx
      .blocking_send(submission)
      .report()
      .change_context(LumatoneMidiError::DeviceSendError)?;
    Ok(response_rx)
  }

  /// Signals to the driver to shutdown the event loop.
  pub async fn done(&self) -> Result<(), LumatoneMidiError> {
    self
      .done_tx
      .send(())
      .await
      .report()
      .change_context(LumatoneMidiError::DeviceSendError)
  }
}

impl MidiDriver {

  /// Creates a new [MidiDriver] targeting the given [LumatoneDevice].
  ///
  /// May fail if unable to connect to the device.
  ///
  /// On success, returns a tuple of (MidiDriver, Future<()>). The
  /// returned future must be `await`ed to start the driver's event loop.
  /// You probably want to spawn a new task for the driver future,
  /// since it will not resolve until you either call [MidiDriver::done]
  /// or an error causes the driver loop to exit.
  // TODO: maybe have this take an already connected LumatoneIO, so we
  // don't need to return a Result.
  pub fn new(
    device: &LumatoneDevice,
  ) -> Result<(MidiDriver, impl Future<Output = ()>), LumatoneMidiError> {
    let internal = MidiDriverInternal::new(device)?;
    let (command_tx, command_rx) = mpsc::channel(128);
    let (done_tx, done_rx) = mpsc::channel(1);

    let driver = MidiDriver {
      command_tx,
      done_tx,
    };
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
      }
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
          // info!("message received, forwarding to state machine");
          self.receive_timeout = None;
          Action::MessageReceived(msg)
        }

        Some(cmd) = commands.recv() => {
          Action::SubmitCommand(cmd)
        }

        _ = done_signal.recv() => {
          debug!("done signal received, exiting");
          return;
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
          error!("effect error: {err}");
          // state = State::Failed(err);
          break;
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

fn to_hex_debug_str(msg: &[u8]) -> String {
  let s = msg
    .iter()
    .map(|b| format!("{b:x}"))
    .collect::<Vec<String>>()
    .join(" ");
  format!("[ {s} ]")
}


mod tests {
  use crate::constants::{CommandId, MANUFACTURER_ID};
  #[allow(unused_imports)]
  use super::*;

  // region State transition tests

  #[test]
  fn submit_command_while_idle_transitions_to_processing_queue() {
    let init = State::Idle;

    let command = Command::Ping(1);
    let (submission, _response_rx) = CommandSubmission::new(command.clone());
    let action = Action::SubmitCommand(submission);

    match init.next(action) {
      State::ProcessingQueue { mut send_queue } => {
        assert_eq!(send_queue.len(), 1);
        let c = send_queue.pop_front().unwrap();
        assert_eq!(c.command, command);
      },
      s => panic!("Unexpected state: {:?}", s)
    }
  }

  #[test]
  fn submit_command_while_awaiting_response_transitions_to_awaiting_response() {
    let cmd1 = Command::Ping(1);
    let cmd2 = Command::Ping(2);

    let (sub1, _) = CommandSubmission::new(cmd1.clone());
    let (sub2, _) = CommandSubmission::new(cmd2.clone());

    let send_queue = VecDeque::from(vec![sub1.clone()]);
    let init = State::AwaitingResponse { send_queue, command_sent: sub1 };
    let action = Action::SubmitCommand(sub2);

    match init.next(action) {
      State::AwaitingResponse { mut send_queue, command_sent } => {
        assert_eq!(send_queue.len(), 2);
        assert_eq!(command_sent.command, cmd1);
        let c2 = send_queue.pop_back().unwrap();
        assert_eq!(c2.command, cmd2);
      },

      s => panic!("Unexpected state: {:?}", s),
    }
  }

  #[test]
  fn submit_command_while_device_busy_transitions_to_device_busy() {
    let cmd1 = Command::Ping(1);
    let cmd2 = Command::Ping(2);

    let (sub1, _) = CommandSubmission::new(cmd1.clone());
    let (sub2, _) = CommandSubmission::new(cmd2.clone());

    let send_queue = VecDeque::from(vec![sub1.clone()]);
    let init = State::WaitingToRetry { send_queue, to_retry: sub1 };
    let action = Action::SubmitCommand(sub2);

    match init.next(action) {
      State::WaitingToRetry { mut send_queue, to_retry } => {
        assert_eq!(send_queue.len(), 2);
        assert_eq!(to_retry.command, cmd1);
        let c2 = send_queue.pop_back().unwrap();
        assert_eq!(c2.command, cmd2);
      },

      s => panic!("Unexpected state: {:?}", s),
    }
  }

  #[test]
  fn submit_command_while_processing_queue_transitions_to_processing_queue() {
    let cmd1 = Command::Ping(1);
    let cmd2 = Command::Ping(2);

    let (sub1, _) = CommandSubmission::new(cmd1.clone());
    let (sub2, _) = CommandSubmission::new(cmd2.clone());

    let send_queue = VecDeque::from(vec![sub1.clone()]);
    let init = State::ProcessingQueue { send_queue };
    let action = Action::SubmitCommand(sub2);

    match init.next(action) {
      State::ProcessingQueue { mut send_queue } => {
        assert_eq!(send_queue.len(), 2);
        let c2 = send_queue.pop_back().unwrap();
        assert_eq!(c2.command, cmd2);
      },

      s => panic!("Unexpected state: {:?}", s),
    }
  }

  #[test]
  fn submit_command_while_processing_response_transitions_to_processing_response() {
    let cmd1 = Command::Ping(1);
    let cmd2 = Command::Ping(2);

    let (sub1, _) = CommandSubmission::new(cmd1.clone());
    let (sub2, _) = CommandSubmission::new(cmd2.clone());

    let send_queue = VecDeque::from(vec![sub1.clone()]);
    let init = State::ProcessingResponse { send_queue, command_sent: sub1, response_msg: vec![] };
    let action = Action::SubmitCommand(sub2);

    match init.next(action) {
      State::ProcessingResponse { mut send_queue, .. } => {
        assert_eq!(send_queue.len(), 2);
        let c2 = send_queue.pop_back().unwrap();
        assert_eq!(c2.command, cmd2);
      },

      s => panic!("Unexpected state: {:?}", s),
    }
  }

  #[test]
  fn message_sent_while_processing_queue_transitions_to_awaiting_response() {
    let cmd1 = Command::Ping(1);
    let cmd2 = Command::Ping(2);

    let (sub1, _) = CommandSubmission::new(cmd1.clone());
    let (sub2, _) = CommandSubmission::new(cmd2.clone());

    let send_queue = VecDeque::from(vec![sub2.clone()]);
    let init = State::ProcessingQueue { send_queue };
    let action = Action::MessageSent(sub1);

    match init.next(action) {
      State::AwaitingResponse { mut send_queue, command_sent } => {
        assert_eq!(send_queue.len(), 1);
        let c2 = send_queue.pop_front().unwrap();
        assert_eq!(c2.command, cmd2);

        assert_eq!(command_sent.command, cmd1);
      },

      s => panic!("Unexpected state: {:?}", s),
    }
  }

  #[test]
  fn message_received_while_awaiting_response_transitions_to_processing_response() {
    let cmd = Command::Ping(1);
    let (sub, _) = CommandSubmission::new(cmd.clone());

    let send_queue = VecDeque::new();
    let init = State::AwaitingResponse { send_queue, command_sent: sub };
    let response: Vec<u8> = vec![0xf0, 0x00];
    let action = Action::MessageReceived(response.clone());

    match init.next(action) {
      State::ProcessingResponse { send_queue, command_sent, response_msg } => {
        assert_eq!(send_queue.len(), 0);
        assert_eq!(command_sent.command, cmd);
        assert_eq!(response_msg, response);
      },

      s => panic!("Unexpected state: {:?}", s),
    }
  }

  #[test]
  fn message_received_while_not_awaiting_response_does_not_transition() {
    let response: Vec<u8> = vec![0xf0, 0x00];

    let init = State::Idle;
    let action = Action::MessageReceived(response);
    match init.next(action) {
      State::Idle => (),
      s => panic!("unexpected state: {:?}", s),
    }
  }

  #[test]
  fn response_dispatched_while_processing_response_transitions_to_processing_queue() {
    let cmd = Command::Ping(1);
    let (sub, _) = CommandSubmission::new(cmd.clone());
    let (sub2, _) = CommandSubmission::new(Command::Ping(2));

    let response: Vec<u8> = vec![0xf0, 0x00];
    let send_queue = VecDeque::from(vec![sub2]);
    let init = State::ProcessingResponse { send_queue, command_sent: sub, response_msg: response.clone() };
    let action = Action::ResponseDispatched;

    match init.next(action) {
      State::ProcessingQueue { send_queue } => {
        assert_eq!(send_queue.len(), 1);
      },

      s => panic!("Unexpected state: {:?}", s),
    }
  }

  #[test]
  fn response_timed_out_while_awaiting_response_transitions_to_processing_queue() {
    let cmd = Command::Ping(1);
    let (sub, _) = CommandSubmission::new(cmd.clone());
    let (sub2, _) = CommandSubmission::new(Command::Ping(2));

    let send_queue = VecDeque::from(vec![sub2]);
    let init = State::AwaitingResponse { send_queue, command_sent: sub };
    let action = Action::ResponseTimedOut;

    match init.next(action) {
      State::ProcessingQueue { send_queue } => {
        assert_eq!(send_queue.len(), 1);
      },

      s => panic!("Unexpected state: {:?}", s),
    }
  }

  #[test]
  fn response_timed_out_while_not_awaiting_response_does_not_transition() {
    let init = State::Idle;
    let action = Action::ResponseTimedOut;
    match init.next(action) {
      State::Idle => (),
      s => panic!("unexpected state: {:?}", s),
    }
  }

  #[test]
  fn ready_to_retry_while_device_busy_transitions_to_processing_queue() {
    let cmd = Command::Ping(1);
    let (sub, _) = CommandSubmission::new(cmd.clone());
    let (sub2, _) = CommandSubmission::new(Command::Ping(2));

    let send_queue = VecDeque::from(vec![sub2]);
    let init = State::WaitingToRetry { send_queue, to_retry: sub };
    let action = Action::ReadyToRetry;

    match init.next(action) {
      State::ProcessingQueue { mut send_queue } => {
        assert_eq!(send_queue.len(), 2);
        let head = send_queue.pop_front().unwrap();
        assert_eq!(head.command, cmd);
      },

      s => panic!("unexpected state: {:?}", s),
    }
  }

  #[test]
  fn ready_to_retry_while_not_device_busy_does_not_transition() {
    let init = State::Idle;
    let action = Action::ReadyToRetry;
    match init.next(action) {
      State::Idle => (),
      s => panic!("unexpected state: {:?}", s),
    }
  }

  #[test]
  fn queue_empty_while_processing_queue_transitions_to_idle() {
    let init = State::ProcessingQueue { send_queue: VecDeque::new() };
    let action = QueueEmpty;
    match init.next(action) {
      State::Idle => (),
      s => panic!("unexpected state: {:?}", s),
    }
  }

  #[test]
  fn queue_empty_while_processing_queue_transitions_to_failed_if_queue_is_non_empty() {
    let cmd = Command::Ping(1);
    let (sub, _) = CommandSubmission::new(cmd.clone());
    let init = State::ProcessingQueue { send_queue: VecDeque::from(vec![sub]) };
    let action = QueueEmpty;
    match init.next(action) {
      State::Failed(_) => (),
      s => panic!("unexpected state: {:?}", s),
    }
  }

  #[test]
  fn undefined_state_transitions_result_in_failed_state() {
    let init = State::Idle;
    let action = Action::ResponseDispatched;
    match init.next(action) {
      State::Failed(_) => (),
      s => panic!("unexpected state: {:?}", s),
    }
  }

  // endregion

  // region State entry tests (for expected Effect and Action)

  #[test]
  fn entering_idle_state_has_no_action_or_effect() {
    let mut s = State::Idle;
    match s.enter() {
      (None, None) => (),
      (e, a) => panic!("unexpected effect or action: ({:?}, {:?})", e, a),
    }
  }

  #[test]
  fn entering_processing_queue_while_queue_returns_no_effect_and_queue_empty_action() {
    let mut s = State::ProcessingQueue { send_queue: VecDeque::new() };
    match s.enter() {
      (None, Some(QueueEmpty)) => (),
      (e, a) => panic!("unexpected effect or action: ({:?}, {:?})", e, a),
    }
  }

  #[test]
  fn entering_processing_queue_while_queue_is_full_returns_send_midi_message_effect_and_message_sent_action() {
    let cmd = Command::Ping(1);
    let (sub, _) = CommandSubmission::new(cmd.clone());
    let send_queue = VecDeque::from(vec![sub]);
    let mut s = State::ProcessingQueue { send_queue };
    match s.enter() {
      (Some(Effect::SendMidiMessage(_)), Some(Action::MessageSent(_))) => (),
      (e, a) => panic!("unexpected effect or action: ({:?}, {:?})", e, a),
    }
  }

  #[test]
  fn entering_device_busy_returns_start_retry_timeout_effect_and_no_action() {
    let cmd = Command::Ping(1);
    let (sub, _) = CommandSubmission::new(cmd.clone());
    let mut s = State::WaitingToRetry { send_queue: VecDeque::new(), to_retry: sub };
    match s.enter() {
      (Some(Effect::StartRetryTimeout), None) => (),
      (e, a) => panic!("unexpected effect or action: ({:?}, {:?})", e, a),
    }
  }

  #[test]
  fn entering_awaiting_response_returns_start_receive_timeout_effect_and_no_action() {
    let cmd = Command::Ping(1);
    let (sub, _) = CommandSubmission::new(cmd.clone());
    let mut s = State::AwaitingResponse { send_queue: VecDeque::new(), command_sent: sub };
    match s.enter() {
      (Some(Effect::StartReceiveTimeout), None) => (),
      (e, a) => panic!("unexpected effect or action: ({:?}, {:?})", e, a),
    }
  }

  // helper fn to return a "pong" response message with a given status code
  fn response_with_status(status: ResponseStatusCode) -> Vec<u8> {
    let mut msg = Vec::from(MANUFACTURER_ID);
    msg.push(0x0); // board index
    msg.push(CommandId::LumaPing.into()); // command id
    msg.push(status.into()); // status byte
    msg.push(0x0); // calibration mode flag
    msg.push(0x0); // remaining zeros are for ping payload
    msg.push(0x0);
    msg.push(0x0);

    msg
  }

  #[test]
  fn entering_processing_response_returns_notify_message_response_effect_and_response_dispatched_action() {
    let cmd = Command::Ping(1);
    let (sub, _) = CommandSubmission::new(cmd.clone());

    let mut s = State::ProcessingResponse {
      send_queue: VecDeque::new(),
      command_sent: sub,
      response_msg: response_with_status(ResponseStatusCode::Ack)
    };

    match s.enter() {
      (Some(Effect::NotifyMessageResponse(_, _)), Some(Action::ResponseDispatched)) => (),
      (e, a) => panic!("unexpected effect or action: ({:?}, {:?})", e, a),
    }

    // TODO: add test cases for the other status codes
  }

  // endregion
}