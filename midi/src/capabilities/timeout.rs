use serde::{Serialize, Deserialize};
use crux_core::capability::Operation;
use crux_core::capability::CapabilityContext;
use crux_macros::Capability;
use uuid::Uuid;

pub type TimeoutId = Uuid;

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub enum TimeoutOperation {
  Set {
    millis: usize,
    timeout_id: TimeoutId,
  },
  Cancel(TimeoutId),
}

impl Operation for TimeoutOperation {
  type Output = TimeoutId;
}

#[derive(Capability)]
pub struct Timeout<Ev> {
  context: CapabilityContext<TimeoutOperation, Ev>,
}

impl<Ev> Timeout<Ev>
  where
    Ev: 'static
{
  pub fn new(context: CapabilityContext<TimeoutOperation, Ev>) -> Self {
    Self { context }
  }

  pub fn set<F>(&self, millis: usize, timeout_id: TimeoutId, make_event: F)
    where F: Fn(TimeoutId) -> Ev + Send
  {
    let ctx = self.context.clone();
    self.context.spawn(async move {
      let op = TimeoutOperation::Set { millis, timeout_id };
      let id = ctx.request_from_shell(op).await;
      let event = make_event(id);
      ctx.update_app(event);
    });
  }

  pub fn cancel<F>(&self, timeout_id: TimeoutId, make_event: F)
    where F: Fn(TimeoutId) -> Ev + Send
  {
    let ctx = self.context.clone();
    self.context.spawn(async move {
      let op = TimeoutOperation::Cancel(timeout_id);
      let id = ctx.request_from_shell(op).await;
      let event = make_event(id);
      ctx.update_app(event);
    });
  }
}
