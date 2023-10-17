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
//!       ┌──────────────►│ Idle │
//!       │               └──┬───┘
//!       │                  │
//!       │                  │ SubmitCommand
//!       │                  │
//!       │  QueueEmpty ┌────▼───────────────┐
//!       └─────────────┤                    │
//!                     │                    │ SubmitCommand
//!     ┌──────────────►│  ProcessingQueue   ◄─────────┐
//!     │    ┌─────────►│                    ┌─────────┘
//!     │    │  ┌──────►│                    │
//!     │    │  │       └────┬───────────────┘
//!     │    │  │            │
//!     │    │  │            │ MessageSent
//!     │    │  │            │
//!     │    │  │       ┌────▼───────────────┐
//!     │    │  │       │                    │ SubmitCommand
//!    ResponseTimedOut │  AwaitingResponse  ◄────────┐
//!     │    │  └───────┤                    ┌────────┘
//!     │    │          └────┬───────────────┘
//!     │    │               │
//!     │    │               │ MessageReceived
//!     │    │               │
//!     │    │          ┌────▼─────────────────┐
//!     │    │          │                      │ SubmitCommand
//!  ResponseDispatched │  ProcessingResponse  ◄────────┐
//!     │    └──────────┤                      ┌────────┘
//!     │               └────┬─────────────────┘
//!     │                    │
//!     │                    │ DeviceBusy
//!     │                    │
//!     │               ┌────▼─────────────────┐
//!     │  ReadyToRetry │                      │ SubmitCommand
//!     └───────────────┤    WaitingToRetry    ◄────────┐
//!                     │                      ┌────────┘
//!                     └──────────────────────┘
//! ```

pub mod state;
pub mod actions;
pub mod effects;
pub mod submission;

#[cfg(test)]
mod tests {
  use std::collections::VecDeque;

  use uuid::Uuid;

  use crate::commands::Command;
  use crate::constants::{CommandId, MANUFACTURER_ID, ResponseStatusCode};

  #[allow(unused_imports)]
  use super::{
    actions::Action,
    effects::Effect,
    state::State,
    submission::CommandSubmission,
  };

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
      }
      s => panic!("Unexpected state: {:?}", s),
    }
  }

  #[test]
  fn submit_command_while_awaiting_response_transitions_to_awaiting_response() {
    let cmd1 = Command::Ping(1);
    let cmd2 = Command::Ping(2);

    let (sub1, _) = CommandSubmission::new(cmd1.clone());
    let (sub2, _) = CommandSubmission::new(cmd2.clone());

    let send_queue = VecDeque::from(vec![sub1.clone()]);
    let init = State::AwaitingResponse {
      send_queue,
      command_sent: sub1,
      timeout_id: Uuid::new_v4(),
    };
    let action = Action::SubmitCommand(sub2);

    match init.next(action) {
      State::AwaitingResponse {
        mut send_queue,
        command_sent,
        timeout_id: Uuid::new_v4(),
      } => {
        assert_eq!(send_queue.len(), 2);
        assert_eq!(command_sent.command, cmd1);
        let c2 = send_queue.pop_back().unwrap();
        assert_eq!(c2.command, cmd2);
      }

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
    let init = State::WaitingToRetry {
      send_queue,
      to_retry: sub1,
      timeout_id: Uuid::new_v4(),
    };
    let action = Action::SubmitCommand(sub2);

    match init.next(action) {
      State::WaitingToRetry {
        mut send_queue,
        to_retry,
        timeout_id: Uuid::new_v4(),
      } => {
        assert_eq!(send_queue.len(), 2);
        assert_eq!(to_retry.command, cmd1);
        let c2 = send_queue.pop_back().unwrap();
        assert_eq!(c2.command, cmd2);
      }

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
      }

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
    let init = State::ProcessingResponse {
      send_queue,
      command_sent: sub1,
      response_msg: vec![],
    };
    let action = Action::SubmitCommand(sub2);

    match init.next(action) {
      State::ProcessingResponse { mut send_queue, .. } => {
        assert_eq!(send_queue.len(), 2);
        let c2 = send_queue.pop_back().unwrap();
        assert_eq!(c2.command, cmd2);
      }

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
      State::AwaitingResponse {
        mut send_queue,
        command_sent,
        timeout_id: Uuid::new_v4(),
      } => {
        assert_eq!(send_queue.len(), 1);
        let c2 = send_queue.pop_front().unwrap();
        assert_eq!(c2.command, cmd2);

        assert_eq!(command_sent.command, cmd1);
      }

      s => panic!("Unexpected state: {:?}", s),
    }
  }

  #[test]
  fn message_received_while_awaiting_response_transitions_to_processing_response() {
    let cmd = Command::Ping(1);
    let (sub, _) = CommandSubmission::new(cmd.clone());

    let send_queue = VecDeque::new();
    let init = State::AwaitingResponse {
      send_queue,
      command_sent: sub,
      timeout_id: Uuid::new_v4(),
    };
    let response: Vec<u8> = vec![0xf0, 0x00];
    let action = Action::MessageReceived(response.clone());

    match init.next(action) {
      State::ProcessingResponse {
        send_queue,
        command_sent,
        response_msg,
      } => {
        assert_eq!(send_queue.len(), 0);
        assert_eq!(command_sent.command, cmd);
        assert_eq!(response_msg, response);
      }

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
    let init = State::ProcessingResponse {
      send_queue,
      command_sent: sub,
      response_msg: response.clone(),
    };
    let action = Action::ResponseDispatched;

    match init.next(action) {
      State::ProcessingQueue { send_queue } => {
        assert_eq!(send_queue.len(), 1);
      }

      s => panic!("Unexpected state: {:?}", s),
    }
  }

  #[test]
  fn response_timed_out_while_awaiting_response_transitions_to_processing_queue() {
    let cmd = Command::Ping(1);
    let (sub, _) = CommandSubmission::new(cmd.clone());
    let (sub2, _) = CommandSubmission::new(Command::Ping(2));

    let send_queue = VecDeque::from(vec![sub2]);
    let init = State::AwaitingResponse {
      send_queue,
      command_sent: sub,
      timeout_id: Uuid::new_v4(),
    };
    let action = Action::ResponseTimedOut;

    match init.next(action) {
      State::ProcessingQueue { send_queue } => {
        assert_eq!(send_queue.len(), 1);
      }

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
    let init = State::WaitingToRetry {
      send_queue,
      to_retry: sub,
      timeout_id: Uuid::new_v4(),
    };
    let action = Action::ReadyToRetry;

    match init.next(action) {
      State::ProcessingQueue { mut send_queue } => {
        assert_eq!(send_queue.len(), 2);
        let head = send_queue.pop_front().unwrap();
        assert_eq!(head.command, cmd);
      }

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
    let init = State::ProcessingQueue {
      send_queue: VecDeque::new(),
    };
    let action = Action::QueueEmpty;
    match init.next(action) {
      State::Idle => (),
      s => panic!("unexpected state: {:?}", s),
    }
  }

  #[test]
  fn queue_empty_while_processing_queue_transitions_to_failed_if_queue_is_non_empty() {
    let cmd = Command::Ping(1);
    let (sub, _) = CommandSubmission::new(cmd.clone());
    let init = State::ProcessingQueue {
      send_queue: VecDeque::from(vec![sub]),
    };
    let action = Action::QueueEmpty;
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

  // region State entry tests (for expected Effect)

  #[test]
  fn entering_idle_state_has_no_effect() {
    let mut s = State::Idle;
    match s.enter() {
      None => (),
      Some(e) => panic!("unexpected effect: {:?}", e),
    }
  }

  #[test]
  fn entering_processing_queue_while_queue_dispatches_queue_empty_action() {
    use Action::QueueEmpty;
    use Effect::DispatchAction;

    let mut s = State::ProcessingQueue {
      send_queue: VecDeque::new(),
    };
    match s.enter() {
      Some(DispatchAction(QueueEmpty)) => (),
      e => panic!("unexpected effect: {:?}", e),
    }
  }

  #[test]
  fn entering_processing_queue_while_queue_is_full_returns_send_midi_message_effect() {
    use Effect::SendMidiMessage;
    use State::ProcessingQueue;

    let cmd = Command::Ping(1);
    let (sub, _) = CommandSubmission::new(cmd.clone());
    let send_queue = VecDeque::from(vec![sub]);
    let mut s = ProcessingQueue { send_queue };
    match s.enter() {
      Some(SendMidiMessage(_)) => (),
      e => panic!("unexpected effect: {:?}", e),
    }
  }

  #[test]
  fn entering_waiting_to_retry_returns_start_retry_timeout_effect() {
    use Effect::StartRetryTimeout;
    use State::WaitingToRetry;

    let cmd = Command::Ping(1);
    let (sub, _) = CommandSubmission::new(cmd.clone());
    let mut s = WaitingToRetry {
      send_queue: VecDeque::new(),
      to_retry: sub,
      timeout_id: Uuid::new_v4(),
    };
    match s.enter() {
      Some(StartRetryTimeout) => (),
      e => panic!("unexpected effect: {:?}", e),
    }
  }

  #[test]
  fn entering_awaiting_response_returns_start_receive_timeout_effect() {
    use Effect::StartReceiveTimeout;
    use State::AwaitingResponse;

    let cmd = Command::Ping(1);
    let (sub, _) = CommandSubmission::new(cmd.clone());
    let mut s = AwaitingResponse {
      send_queue: VecDeque::new(),
      command_sent: sub,
      timeout_id: Uuid::new_v4(),
    };
    match s.enter() {
      Some(StartReceiveTimeout) => (),
      e => panic!("unexpected effect: {:?}", e),
    }
  }

  // helper fn to return a "pong" response message with a given status code
  #[allow(dead_code)]
  fn response_with_status(status: ResponseStatusCode) -> Vec<u8> {
    let mut msg = Vec::from(MANUFACTURER_ID);
    msg.push(0x0); // board index
    msg.push(CommandId::LumaPing.into()); // command id
    msg.push(status.into()); // status byte
    msg.push(0x7f); // "echo" flag - must be set to 0x7f for ping response
    msg.push(0x0); // remaining zeros are ping id payload
    msg.push(0x0);
    msg.push(0x0);

    msg
  }

  #[test]
  fn entering_processing_response_with_status_ack_returns_ok_notify_message_response_effect() {
    use Effect::NotifyMessageResponse;
    use State::ProcessingResponse;

    let cmd = Command::Ping(1);
    let (sub, _) = CommandSubmission::new(cmd.clone());

    let mut s = ProcessingResponse {
      send_queue: VecDeque::new(),
      command_sent: sub,
      response_msg: response_with_status(ResponseStatusCode::Ack),
    };

    match s.enter() {
      Some(NotifyMessageResponse(_, Ok(_))) => (),
      e => panic!("unexpected effect: {:?}", e),
    }
  }

  #[test]
  fn entering_processing_response_with_status_nack_returns_err_notify_message_response_effect() {
    use Effect::NotifyMessageResponse;
    use State::ProcessingResponse;

    let cmd = Command::Ping(1);
    let (sub, _) = CommandSubmission::new(cmd.clone());

    let mut s = ProcessingResponse {
      send_queue: VecDeque::new(),
      command_sent: sub,
      response_msg: response_with_status(ResponseStatusCode::Nack),
    };

    match s.enter() {
      Some(NotifyMessageResponse(_, Err(_))) => (),
      e => panic!("unexpected effect: {:?}", e),
    }
  }

  #[test]
  fn entering_processing_response_with_status_error_returns_err_notify_message_response_effect() {
    use Effect::NotifyMessageResponse;
    use State::ProcessingResponse;

    let cmd = Command::Ping(1);
    let (sub, _) = CommandSubmission::new(cmd.clone());

    let mut s = ProcessingResponse {
      send_queue: VecDeque::new(),
      command_sent: sub,
      response_msg: response_with_status(ResponseStatusCode::Error),
    };

    match s.enter() {
      Some(NotifyMessageResponse(_, Err(_))) => (),
      e => panic!("unexpected effect: {:?}", e),
    }
  }

  #[test]
  fn entering_processing_response_with_status_busy_dispatches_device_busy_action() {
    use Action::DeviceBusy;
    use Effect::DispatchAction;
    use State::ProcessingResponse;

    let cmd = Command::Ping(1);
    let (sub, _) = CommandSubmission::new(cmd.clone());

    let mut s = ProcessingResponse {
      send_queue: VecDeque::new(),
      command_sent: sub,
      response_msg: response_with_status(ResponseStatusCode::Busy),
    };

    match s.enter() {
      Some(DispatchAction(DeviceBusy)) => (),
      e => panic!("unexpected effect: {:?}", e),
    }
  }

  #[test]
  fn entering_processing_response_with_status_state_dispatches_device_busy_action() {
    use Action::DeviceBusy;
    use Effect::DispatchAction;
    use State::ProcessingResponse;

    let cmd = Command::Ping(1);
    let (sub, _) = CommandSubmission::new(cmd.clone());

    let mut s = ProcessingResponse {
      send_queue: VecDeque::new(),
      command_sent: sub,
      response_msg: response_with_status(ResponseStatusCode::State),
    };

    match s.enter() {
      Some(DispatchAction(DeviceBusy)) => (),
      e => panic!("unexpected effect: {:?}", e),
    }
  }

  #[test]
  fn entering_processing_response_with_status_unknown_returns_no_effect() {
    let cmd = Command::Ping(1);
    let (sub, _) = CommandSubmission::new(cmd.clone());

    let mut s = State::ProcessingResponse {
      send_queue: VecDeque::new(),
      command_sent: sub,
      response_msg: response_with_status(ResponseStatusCode::Unknown),
    };

    match s.enter() {
      None => (),
      e => panic!("unexpected effect: {:?}", e),
    }
  }

  // endregion
}
