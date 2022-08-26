#![allow(dead_code)]

use log::warn;
use midir::{MidiIO, MidiInput, MidiInputConnection, MidiOutput, MidiOutputConnection};
use tokio::sync::mpsc;

use super::{error::LumatoneMidiError, sysex::EncodedSysex};
use error_stack::{report, IntoReport, Result, ResultExt};

/// Identifies the MIDI input and output ports that the Lumatone is connected to.
/// A LumatoneDevice can be used to initiate a connection to the device using [`Self::connect`].
#[derive(Debug, Clone)]
pub struct LumatoneDevice {
  out_port_name: String,
  in_port_name: String,
}

impl LumatoneDevice {
  pub fn new(output_port_name: &str, input_port_name: &str) -> LumatoneDevice {
    LumatoneDevice {
      out_port_name: output_port_name.to_string(),
      in_port_name: input_port_name.to_string(),
    }
  }

  /// Connects to the MIDI ports for this LumatoneDevice.
  /// Returns a [`LumatoneIO`] on success.
  pub fn connect(&self) -> Result<LumatoneIO, LumatoneMidiError> {
    use LumatoneMidiError::DeviceConnectionError;

    let client_name = "lumatone-rs";
    let input = MidiInput::new(client_name)
      .report()
      .change_context(DeviceConnectionError)?;
    let output = MidiOutput::new(client_name)
      .report()
      .change_context(DeviceConnectionError)?;

    let in_port =
      get_port_by_name(&input, &self.in_port_name).change_context(DeviceConnectionError)?;
    let out_port =
      get_port_by_name(&output, &self.out_port_name).change_context(DeviceConnectionError)?;

    let buf_size = 32;
    let (incoming_tx, incoming_messages) = mpsc::channel(buf_size);

    let input_conn = input
      .connect(
        &in_port,
        &self.in_port_name,
        move |_, msg, _| {
          let msg = msg.to_vec();
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

    let output_conn = output.connect(&out_port, &self.out_port_name).map_err(|e|
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
}

/// Represents an open connection to a Lumatone device that can send and receive messages.
pub struct LumatoneIO {
  input_conn: MidiInputConnection<()>,
  output_conn: MidiOutputConnection,

  /// All incoming MIDI messages will be pushed onto this channel.
  // TODO: should this be a broadcast instead?
  pub incoming_messages: mpsc::Receiver<EncodedSysex>,
}

impl LumatoneIO {
  /// Sends an encoded sysex message to the Lumatone.
  pub fn send(&mut self, msg: &[u8]) -> Result<(), LumatoneMidiError> {
    self
      .output_conn
      .send(msg)
      .report()
      .change_context(LumatoneMidiError::DeviceSendError)
  }

  /// Closes MIDI connections and consumes `self`, making this LumatoneIO unusable.
  /// A new connection can be established using [`LumatoneDevice::connect`].
  pub fn close(self) {
    self.input_conn.close();
    self.output_conn.close();
  }
}

fn get_port_by_name<IO: MidiIO>(io: &IO, name: &str) -> Result<IO::Port, LumatoneMidiError> {
  for p in io.ports() {
    let port_name = io.port_name(&p).map_err(|e| {
      report!(LumatoneMidiError::DeviceConnectionError)
        .attach_printable(format!("unable to get port with name '{name}': {e}"))
    })?;
    if port_name == name {
      return Ok(p);
    }
  }
  Err(
    report!(LumatoneMidiError::DeviceConnectionError)
      .attach_printable(format!("unable to get port with name: {name}")),
  )
}
