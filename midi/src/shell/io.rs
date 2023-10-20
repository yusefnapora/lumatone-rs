use midir::{MidiInputConnection, MidiOutputConnection};
use tokio::sync::mpsc;
use crate::error::LumatoneMidiError;
use crate::sysex::EncodedSysex;

/// Represents an open connection to a Lumatone device that can send and receive messages.
pub struct LumatoneIO {
  pub input_conn: MidiInputConnection<()>,
  pub output_conn: MidiOutputConnection,

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
      .map_err(|e| LumatoneMidiError::DeviceSendError(format!("send error: {e}")))
  }

  /// Closes MIDI connections and consumes `self`, making this LumatoneIO unusable.
  /// A new connection can be established using [`connect`].
  pub fn close(self) {
    self.input_conn.close();
    self.output_conn.close();
  }
}


