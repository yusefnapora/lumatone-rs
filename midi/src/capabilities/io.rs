use serde::{Serialize, Deserialize};
use crux_core::capability::Operation;
use crux_core::capability::CapabilityContext;
use crux_macros::Capability;
use crate::error::LumatoneMidiError;
use crate::sysex::EncodedSysex;
use futures::StreamExt;
use crate::capabilities::connect::DeviceConnectionId;

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct SendSysexOperation {
  connection_id: DeviceConnectionId,
  message: EncodedSysex,
}


impl Operation for SendSysexOperation {
  type Output = Result<(), LumatoneMidiError>;
}

#[derive(Capability)]
pub struct SendSysex<Ev> {
  context: CapabilityContext<SendSysexOperation, Ev>,
}

impl<Ev> SendSysex<Ev>
  where
    Ev: 'static
{
  pub fn new(context: CapabilityContext<SendSysexOperation, Ev>) -> Self {
    Self { context }
  }

  pub fn send<F>(&self, connection_id: DeviceConnectionId, message: EncodedSysex, event: F)
    where F: Fn(Result<(), LumatoneMidiError>) -> Ev + Send + 'static
  {
    let ctx = self.context.clone();
    self.context.spawn(async move {
      let op = SendSysexOperation { connection_id, message };
      let response = ctx.request_from_shell(op).await;
      let ev = event(response);
      ctx.update_app(ev);
    });
  }
}


#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct ReceiveSysexOperation {
  connection_id: DeviceConnectionId
}

pub struct IncomingSysex {
  message: EncodedSysex,
  connection_id: DeviceConnectionId,
}

impl Operation for ReceiveSysexOperation {
  type Output = EncodedSysex;
}

#[derive(Capability)]
pub struct ReceiveSysexStream<Ev> {
  context: CapabilityContext<ReceiveSysexOperation, Ev>,
}

impl<Ev> ReceiveSysexStream<Ev>
  where Ev: 'static
{
  pub fn new(context: CapabilityContext<ReceiveSysexOperation, Ev>) -> Self {
    Self { context }
  }

  pub fn receive<F>(&self, connection_id: DeviceConnectionId, make_event: F)
    where F: Fn(IncomingSysex) -> Ev + Send + Clone + 'static
  {
    let ctx = self.context.clone();
    self.context.spawn(async move {
      let op = ReceiveSysexOperation { connection_id };
      let mut stream = ctx.stream_from_shell(op);

      while let Some(message) = stream.next().await {
        let make_event = make_event.clone();
        let ev = make_event(IncomingSysex { message, connection_id });
        ctx.update_app(ev);
      }
    });
  }
}