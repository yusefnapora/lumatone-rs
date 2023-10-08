use error_stack::{Result, IntoReport, ResultExt, report};
use midir::{MidiIO, MidiInputConnection, MidiOutputConnection};
use tokio::sync::mpsc;
use crate::error::LumatoneMidiError;
use crate::sysex::EncodedSysex;

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
      .into_report()
      .change_context(LumatoneMidiError::DeviceSendError)
  }

  /// Closes MIDI connections and consumes `self`, making this LumatoneIO unusable.
  /// A new connection can be established using [`connect`].
  pub fn close(self) {
    self.input_conn.close();
    self.output_conn.close();
  }
}

fn get_port_by_name<IO: MidiIO, S: AsRef<str>>(io: &IO, name: S) -> Result<IO::Port, LumatoneMidiError> {
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
