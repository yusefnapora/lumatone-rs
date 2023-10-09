use std::fmt::Display;
use crate::driver::submission::CommandSubmission;
use crate::sysex::{EncodedSysex, to_hex_debug_str};

/// Actions are inputs into the state machine.
/// An Action may trigger a state transition, but not all actions are applicable to all states.
/// See the code of [`State::next`] for the valid (action, state) pairings.
#[derive(Debug, Clone)]
pub enum Action {
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