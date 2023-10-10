use std::fmt::{Display, Debug};
use serde::{Serialize, Deserialize};
use uuid::Uuid;
use crate::commands::Command;

pub type CommandSubmissionId = Uuid;

/// Request to send a command to the device, with a unique submission id used to correlate
/// responses with command submissions.
#[derive(Serialize, Deserialize, PartialEq, Eq, Clone, Debug)]
pub struct CommandSubmission {
  pub command: Command,
  pub submission_id: CommandSubmissionId,
}

impl CommandSubmission {
  pub fn new(command: Command) -> Self {
    CommandSubmission {
      command,
      submission_id: Uuid::new_v4(),
    }
  }
}
impl Display for CommandSubmission {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    write!(f, "CommandSubmission({})", self.command)
  }
}