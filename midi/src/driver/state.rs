use std::collections::VecDeque;
use std::fmt::Display;
use error_stack::{Report, report};
use log::{debug, error, warn};
use uuid::Uuid;
use crate::capabilities::timeout::TimeoutId;
use crate::commands::Command;
use crate::constants::ResponseStatusCode;
use crate::driver::actions::Action;
use crate::driver::effects::Effect;
use crate::driver::submission::CommandSubmission;
use crate::error::LumatoneMidiError;
use crate::responses::Response;
use crate::sysex::{EncodedSysex, is_response_to_message, message_answer_code, to_hex_debug_str};

/// One of the possible states the MIDI driver can be in at any given time.
#[derive(Debug)]
pub enum State {
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
    timeout_id: TimeoutId,
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
    timeout_id: TimeoutId,
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
        ..
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
        ..
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


impl State {
  /// Applies an [Action] to the current [State] and returns the new State.
  /// Note that this may be the same as the original state, in cases where the given
  /// Action does not apply to the current state.
  pub(crate) fn next(self, action: Action) -> State {
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
          timeout_id,
        },
      ) => {
        // add new command to the send_queue
        send_queue.push_back(cmd);
        AwaitingResponse {
          send_queue,
          command_sent,
          timeout_id,
        }
      }

      // Submitting a commmand while we're waiting to retry a previous command transitions to a new
      // WaitingToRetry state with the new command pushed onto the send queue.
      (
        SubmitCommand(cmd),
        WaitingToRetry {
          mut send_queue,
          to_retry,
          timeout_id,
        },
      ) => {
        // add new command to the send queue
        send_queue.push_back(cmd);
        WaitingToRetry {
          send_queue,
          to_retry,
          timeout_id,
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
      (
        SubmitCommand(cmd),
        ProcessingResponse {
          mut send_queue,
          command_sent,
          response_msg,
        },
      ) => {
        send_queue.push_back(cmd);
        ProcessingResponse {
          send_queue,
          command_sent,
          response_msg,
        }
      }

      // Getting confirmation that a message was sent out while we're processing the queue transitions to
      // the AwaitingResponse state.
      (MessageSent(command_sent), ProcessingQueue { send_queue }) => AwaitingResponse {
        send_queue,
        command_sent,
        // FIXME: just generating the timeout_id here to get things compiling. need to set real timeout via capability
        timeout_id: Uuid::new_v4()
      },

      // Receiving a message when we're awaiting a response transitions to ProcessingResponse
      (
        MessageReceived(response_msg),
        AwaitingResponse {
          send_queue,
          command_sent,
          .. // TODO: request timeout cancellation
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
      (ResponseDispatched, ProcessingResponse { send_queue, .. }) => ProcessingQueue { send_queue },

      // Getting a DeviceBusy signal when we're processing a response transitions to WaitingToRetry
      (
        DeviceBusy,
        ProcessingResponse {
          send_queue,
          command_sent,
          ..
        },
      ) => WaitingToRetry {
        send_queue,
        to_retry: command_sent,
        // FIXME: just generating the timeout_id here to get things compiling. need to set real timeout via capability
        timeout_id: Uuid::new_v4()
      },

      // Getting a ResponseTimedOut action while waiting for a response logs a warning
      // and transitions to ProcessingQueue.
      // TODO: this should retry or return a failure on the response channel instead of ignoring
      (
        ResponseTimedOut,
        AwaitingResponse {
          send_queue,
          command_sent,
          ..
        },
      ) => {
        warn!("Timed out waiting for response to msg: {:?}", command_sent);
        ProcessingQueue { send_queue }
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
          ..
        },
      ) => {
        send_queue.push_front(to_retry);
        ProcessingQueue { send_queue }
      }

      // Getting a QueueEmpty action when we're in the ProcessingQueue state transitions to Idle.
      // If the queue is not actually empty, transitions to Failed, as that shouldn't happen
      (QueueEmpty, ProcessingQueue { send_queue }) => {
        if !send_queue.is_empty() {
          let msg = format!(
            "Received QueueEmpty action, but queue has {} elements",
            send_queue.len()
          );
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
  pub(crate) fn enter(&mut self) -> Option<Effect> {
    use Effect::*;
    use State::*;

    // debug!("entering state {:?}", self);

    match self {
      Idle => None,
      ProcessingQueue { send_queue } => match send_queue.pop_front() {
        None => Some(DispatchAction(Action::QueueEmpty)),
        Some(cmd) => Some(SendMidiMessage(cmd.clone())),
      },
      WaitingToRetry { .. } => Some(StartRetryTimeout),
      AwaitingResponse { .. } => Some(StartReceiveTimeout),
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
          ResponseStatusCode::Busy => Some(DispatchAction(Action::DeviceBusy)),

          ResponseStatusCode::State => {
            warn!("device is in demo mode!");
            // FIXME: demo mode should probably have its own action that triggers
            // sending a command to exit demo mode.
            Some(DispatchAction(Action::DeviceBusy))
          }

          ResponseStatusCode::Error => {
            let res = Err(report!(LumatoneMidiError::InvalidResponseMessage(
              "Device response had error flag set".to_string()
            )));
            let effect = NotifyMessageResponse(command_sent.clone(), res);
            Some(effect)
          }

          ResponseStatusCode::Nack => {
            let res = Err(report!(LumatoneMidiError::InvalidResponseMessage(format!(
              "Device sent NACK in response to command {command_sent:?}"
            ))));
            let effect = NotifyMessageResponse(command_sent.clone(), res);
            Some(effect)
          }

          ResponseStatusCode::Ack => {
            let response_res = Response::from_sysex_message(response_msg);

            let effect = NotifyMessageResponse(command_sent.clone(), response_res);
            Some(effect)
          }

          ResponseStatusCode::Unknown => {
            // Unknown means the device sent a status code we don't know about.
            // log a warning and pretend it's all good
            warn!("Unknown response status code received");
            None
          }
        }
      }
      Failed(err) => {
        error!("midi driver - unrecoverable error: {err}");
        None // todo: return ExitWithError effect
      }
    }
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