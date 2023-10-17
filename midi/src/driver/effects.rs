use std::fmt::Display;
use crate::driver::actions::Action;
use crate::driver::submission::CommandSubmission;
use crate::error::LumatoneMidiError;
use crate::responses::Response;


/// Effects are requests from the state machine to "do something" in the outside world.
#[derive(Debug)]
pub enum Effect {
  /// The state machine has a message ready to send on the MIDI out port.
  SendMidiMessage(CommandSubmission),

  /// The state machine wants to start the receive timeout.
  StartReceiveTimeout,

  /// The state machine wants to start the busy/retry timeout.
  StartRetryTimeout,

  /// The state machine has received a response to a message and wants to notify
  /// the outside world about its success or failure.
  NotifyMessageResponse(CommandSubmission, Result<Response, LumatoneMidiError>),

  /// The [State] we just [enter](State::enter)ed wants to transition to a new state,
  /// and we should feed the given [Action] into the state machine next.
  DispatchAction(Action),
}

impl Display for Effect {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    use Effect::*;
    match self {
      SendMidiMessage(cmd) => write!(f, "SendMidiMessage({})", cmd.command),
      StartReceiveTimeout => write!(f, "StartReceiveTimeout"),
      StartRetryTimeout => write!(f, "StartRetryTimeout"),
      NotifyMessageResponse(cmd, res) => {
        write!(f, "NotifyMessageResponse({}, {:?})", cmd.command, res)
      }
      DispatchAction(action) => write!(f, "DispatchAction({})", action),
    }
  }
}
