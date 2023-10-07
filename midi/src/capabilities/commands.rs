use serde::{Serialize, Deserialize};
use crux_macros::Capability;
use crux_core::capability::{Capability, CapabilityContext, Operation};
use crate::commands::Command;

/// When the shell submits a Lumatone command to the core, the core
/// will respond with a unique `CommandSubmissionId` that will be
/// included in the command's response event.
type CommandSubmissionId = String;

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct SendLumatoneCommandOperation {
    command: Command
}

impl Operation for SendLumatoneCommandOperation {
    type Output = CommandSubmissionId;
}

#[derive(Capability)]
pub struct SendCommand<Ev> {
    context: CapabilityContext<SendLumatoneCommandOperation, Ev>,
}

impl<Ev> SendCommand<Ev> {
    pub fn new(context: CapabilityContext<SendLumatoneCommandOperation, Ev>) -> Self {
        Self { context }
    }
}
