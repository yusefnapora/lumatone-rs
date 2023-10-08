use serde::{Serialize, Deserialize};
use crux_core::capability::Operation;
use crux_core::capability::CapabilityContext;
use crux_macros::Capability;
use crate::error::LumatoneMidiError;
use crate::sysex::EncodedSysex;
use futures::StreamExt;

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub enum SysexOperation {
  Send(EncodedSysex),
  Listen
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub enum SysexOutput {
  SendResult(Result<(), LumatoneMidiError>),
  IncomingMessage(EncodedSysex),
}

impl Operation for SysexOperation {
  type Output = SysexOutput;
}

#[derive(Capability)]
pub struct Sysex<Ev> {
  context: CapabilityContext<SysexOperation, Ev>,
}

impl<Ev> Sysex<Ev>
  where
    Ev: 'static
{
  pub fn new(context: CapabilityContext<SysexOperation, Ev>) -> Self {
    Self { context }
  }

  pub fn send<F>(&self, message: EncodedSysex, event: F)
    where F: Fn(Result<(), LumatoneMidiError>) -> Ev + Send + 'static
  {
    let ctx = self.context.clone();
    self.context.spawn(async move {
      let op = SysexOperation::Send(message);
      let SysexOutput::SendResult(response) = ctx.request_from_shell(op).await else {
        panic!("expected operation output to be SysexOutput::SendResult")
      };
      let ev = event(response);
      ctx.update_app(ev);
    });
  }

  pub fn receive<F>(&self, make_event: F)
    where F: Fn(EncodedSysex) -> Ev + Send + Clone + 'static
  {
    let ctx = self.context.clone();
    self.context.spawn(async move {
      let op = SysexOperation::Listen;
      let mut stream = ctx.stream_from_shell(op);

      while let Some(output) = stream.next().await {
        let SysexOutput::IncomingMessage(message) = output else {
          // TODO: log a warning here?
          continue
        };
        let make_event = make_event.clone();
        let ev = make_event(message);
        ctx.update_app(ev);
      }
    });
  }
}