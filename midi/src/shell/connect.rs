#![allow(dead_code)]

use log::{debug, warn};
use midir::{ MidiInput, MidiOutput };
use tokio::sync::mpsc;
use error_stack::{report, IntoReport, Result, ResultExt};

use crate::{error::LumatoneMidiError, sysex::SYSEX_START};
use super::io::LumatoneIO;



/// Connects to a lumatone device on the given input and output ports
/// Returns a [`LumatoneIO`] on success.
pub fn connect<S: AsRef<str>>(input_name: S, output_name: S) -> Result<LumatoneIO, LumatoneMidiError> {
  use LumatoneMidiError::DeviceConnectionError;

  let client_name = "lumatone-rs";
  let input = MidiInput::new(client_name)
    .into_report()
    .change_context(DeviceConnectionError)?;
  let output = MidiOutput::new(client_name)
    .into_report()
    .change_context(DeviceConnectionError)?;

  let in_port =
    get_port_by_name(&input, &*input_name).change_context(DeviceConnectionError)?;
  let out_port =
    get_port_by_name(&output, &*output_name).change_context(DeviceConnectionError)?;

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
    .map_err(|e|
      // The ConnectError<MidiInput> type is not thread-safe, so we stringify instead of report()-ing directly
      report!(DeviceConnectionError)
        .attach_printable(format!("midi input connection error: {e}")))?;

  let output_conn = output.connect(&out_port, &*output_name).map_err(|e|
    // The ConnectError<MidiOutput> type is not thread-safe, so we stringify instead of report()-ing directly
    report!(DeviceConnectionError)
      .attach_printable(format!("midi input connection error: {e}")))?;

  let io = LumatoneIO {
    input_conn,
    output_conn,
    incoming_messages,
  };
  Ok(io)
}

