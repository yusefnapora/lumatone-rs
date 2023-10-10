use serde::{Serialize, Deserialize};
use crux_core::capability::Operation;
use crux_core::capability::CapabilityContext;
use crux_macros::Capability;
use uuid::Uuid;
use crate::error::LumatoneMidiError;
use crate::responses::Response;

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub enum NotificationOperation {
	CommandResult {
	  result: Result<Response, LumatoneMidiError>,
		submission_id: Uuid,
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

  pub fn send_command_result(&self, submission_id: Uuid, result: Result<Response, LumatoneMidiError>)
  {
    let ctx = self.context.clone();
    self.context.spawn(async move {
			let op = NotificationOperation::CommandResult { submission_id, result };
      ctx.request_from_shell(op).await;
    });
  }
}
