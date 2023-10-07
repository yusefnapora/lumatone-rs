use serde::{Serialize, Deserialize};
use crux_core::capability::Operation;
use crux_core::capability::CapabilityContext;
use crux_macros::Capability;
use uuid::Uuid;
use crate::capabilities::detect::LumatoneDeviceDescriptor;
use crate::error::LumatoneMidiError;


#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct ConnectToDeviceOperation {
  device: LumatoneDeviceDescriptor
}

pub type DeviceConnectionId = Uuid;

impl Operation for ConnectToDeviceOperation {
  type Output = Result<DeviceConnectionId, LumatoneMidiError>;
}


#[derive(Capability)]
pub struct ConnectToDevice<Ev> {
  context: CapabilityContext<ConnectToDeviceOperation, Ev>,
}

impl<Ev> ConnectToDevice<Ev>
  where
    Ev: 'static
{
  pub fn new(context: CapabilityContext<ConnectToDeviceOperation, Ev>) -> Self {
    Self { context }
  }

  pub fn connect<F>(&self, device: LumatoneDeviceDescriptor, event: F)
    where F: Fn(Result<DeviceConnectionId, LumatoneMidiError>) -> Ev + Send + 'static
  {
    let ctx = self.context.clone();
    self.context.spawn(async move {
      let op = ConnectToDeviceOperation { device };
      let response = ctx.request_from_shell(op).await;
      let ev = event(response);
      ctx.update_app(ev);
    });
  }
}
