use serde::{Serialize, Deserialize};
use crux_core::capability::Operation;
use crux_core::capability::CapabilityContext;
use crux_macros::Capability;


#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct DetectDeviceOperation;

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct LumatoneDeviceDescriptor {
  id: String,
  input_id: String,
  output_id: String,
}


impl Operation for DetectDeviceOperation {
  // FIXME: should be a Result with an error type
  type Output = LumatoneDeviceDescriptor;
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
    where F: Fn(LumatoneDeviceDescriptor) -> Ev + Send + 'static
  {
    let ctx = self.context.clone();
    self.context.spawn(async move {
      let response = ctx.request_from_shell(DetectDeviceOperation).await;
      let ev = event(response);
      ctx.update_app(ev);
    });
  }
}
