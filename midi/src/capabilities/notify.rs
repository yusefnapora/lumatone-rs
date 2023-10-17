use serde::{Serialize, Deserialize};
use crux_core::capability::Operation;
use crux_core::capability::CapabilityContext;
use crux_macros::Capability;
use crate::error::LumatoneMidiError;
use crate::responses::Response;
use crate::driver::submission::CommandSubmissionId;

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub enum NotificationOperation {
	CommandResult {
	  result: Result<Response, LumatoneMidiError>,
		submission_id: CommandSubmissionId,
	}
}


impl Operation for NotificationOperation {
  type Output = ();
}


#[derive(Capability)]
pub struct NotifyShell<Ev> {
  context: CapabilityContext<NotificationOperation, Ev>,
}

impl<Ev> NotifyShell<Ev>
  where
      Ev: 'static
{
  pub fn new(context: CapabilityContext<NotificationOperation, Ev>) -> Self {
    Self { context }
  }

  pub fn send_command_result(&self, submission_id: CommandSubmissionId, result: Result<Response, LumatoneMidiError>)
  {
    let ctx = self.context.clone();
    self.context.spawn(async move {
			let op = NotificationOperation::CommandResult { submission_id, result };
      ctx.request_from_shell(op).await;
    });
  }
}
