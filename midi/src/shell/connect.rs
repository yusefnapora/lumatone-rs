#![allow(dead_code)]

use log::{debug, warn};
use midir::{ MidiInput, MidiOutput };
use tokio::sync::mpsc;

use crate::{error::LumatoneMidiError, sysex::SYSEX_START};
use super::io::LumatoneIO;



/// Connects to a lumatone device on the given input and output ports
/// Returns a [`LumatoneIO`] on success.
pub fn connect<S: AsRef<str>>(input_name: S, output_name: S) -> Result<LumatoneIO, LumatoneMidiError> {
  use LumatoneMidiError::DeviceConnectionError;

  let client_name = "lumatone-rs";
  let input = MidiInput::new(client_name)
    .map_err(|e| DeviceConnectionError(format!("error creating MidiInput: {e}")))?;
  let output = MidiOutput::new(client_name)
    .map_err(|e| DeviceConnectionError(format!("error creating MidiOutput: {e}")))?;

  let in_port =
    get_port_by_name(&input, &*input_name)?;
  let out_port =
    get_port_by_name(&output, &*output_name)?;

  let buf_size = 32;
  let (incoming_tx, incoming_messages) = mpsc::channel(buf_size);

  let input_conn = input
    .connect(
      &in_port,
      &*input_name,
      move |_, msg, _| {
        let msg = msg.to_vec();
        if msg.is_empty() || msg[0] != SYSEX_START {
          debug!("received non sysex message, ignoring");
          return;
        }
        if let Err(err) = incoming_tx.blocking_send(msg) {
          warn!("error sending incoming message on channel: {err}");
        }
      },
      (),
    )
    .map_err(|e| DeviceConnectionError(format!("output connection error: {e}")))?;

  let output_conn = output.connect(&out_port, &*output_name).map_err(|e|
    DeviceConnectionError(format!("midi input connection error: {e}")))?;

  let io = LumatoneIO {
    input_conn,
    output_conn,
    incoming_messages,
  };
  Ok(io)
}

fn get_port_by_name<IO: MidiIO, S: AsRef<str>>(io: &IO, name: S) -> Result<IO::Port, LumatoneMidiError> {
  for p in io.ports() {
    let port_name = io.port_name(&p).map_err(|e| {
      LumatoneMidiError::DeviceConnectionError(format!("unable to get port with name '{name}': {e}"))
    })?;
    if port_name == name {
      return Ok(p);
    }
  }
  Err(
    LumatoneMidiError::DeviceConnectionError(format!("unable to get port with name: {name}")),
  )
}
