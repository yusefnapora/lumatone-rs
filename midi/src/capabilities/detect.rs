use serde::{Serialize, Deserialize};
use crux_core::capability::Operation;
use crux_core::capability::CapabilityContext;
use crux_macros::Capability;
use crate::error::LumatoneMidiError;


#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct DetectDeviceOperation;

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct LumatoneDeviceDescriptor {
  id: String,
  input_id: String,
  output_id: String,
}


impl Operation for DetectDeviceOperation {
  type Output = Result<LumatoneDeviceDescriptor, LumatoneMidiError>;
}


#[derive(Capability)]
pub struct DetectDevice<Ev> {
  context: CapabilityContext<DetectDeviceOperation, Ev>,
}

impl<Ev> DetectDevice<Ev>
  where
      Ev: 'static
{
  pub fn new(context: CapabilityContext<DetectDeviceOperation, Ev>) -> Self {
    Self { context }
  }

  pub fn detect<F>(&self, event: F)
    where F: Fn(Result<LumatoneDeviceDescriptor, LumatoneMidiError>) -> Ev + Send + 'static
  {
    let ctx = self.context.clone();
    self.context.spawn(async move {
      let response = ctx.request_from_shell(DetectDeviceOperation).await;
      let ev = event(response);
      ctx.update_app(ev);
    });
  }
}
