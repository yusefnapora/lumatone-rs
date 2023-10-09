use std::fmt::{Display, Debug};
use tokio::sync::mpsc;
use crate::commands::Command;
use crate::error::LumatoneMidiError;
use crate::responses::Response;

/// Result type returned in response to a command submission
pub type ResponseResult = Result<Response, LumatoneMidiError>;

/// Request to send a command to the device, with a unique submission id used to correlate
/// responses with command submissions.
#[derive(Clone)]
pub struct CommandSubmission {
  pub command: Command,
  pub response_tx: mpsc::Sender<ResponseResult>,
}

impl CommandSubmission {
  /// Creates a new CommandSubmission and returns it, along with the receive channel
  /// for the command's [ResponseResult].
  pub(crate) fn new(command: Command) -> (Self, mpsc::Receiver<ResponseResult>) {
    let (response_tx, response_rx) = mpsc::channel(1);
    let sub = CommandSubmission {
      command,
      response_tx,
    };
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