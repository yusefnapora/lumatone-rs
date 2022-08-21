use super::constants::CommandId;

use midir::{ConnectError, InitError, MidiInput, MidiOutput, PortInfoError, SendError};
use std::fmt::Display;

#[derive(Debug)]
pub enum LumatoneMidiError {
  // InvalidCommandInput(CommandId, String),
  NotLumatoneMessage(Vec<u8>),
  MessageTooShort {
    expected: usize,
    actual: usize,
  },
  MessagePayloadTooShort {
    expected: usize,
    actual: usize,
  },
  UnknownCommandId(u8),
  UnexpectedCommandId {
    expected: CommandId,
    actual: CommandId,
  },
  UnsupportedCommandId(CommandId, String),
  InvalidResponseMessage(String),

  MidiPortNotFound(String),
  MidiPortInfoError(PortInfoError),
  MidiInitError(InitError),
  MidiSendError(SendError),
  MidiInputConnectError(ConnectError<MidiInput>),
  MidiOutputConnectError(ConnectError<MidiOutput>),

  InvalidStateTransition(String),
  DeviceDetectionFailed(String),
  InvalidBoardIndex(u8),
  InvalidMidiChannel(u8),
  InvalidLumatoneKeyIndex(u8),
  InvalidPresetIndex(u8),
}

impl From<InitError> for LumatoneMidiError {
  fn from(e: InitError) -> Self {
    LumatoneMidiError::MidiInitError(e)
  }
}

impl From<SendError> for LumatoneMidiError {
  fn from(e: SendError) -> Self {
    LumatoneMidiError::MidiSendError(e)
  }
}

impl From<PortInfoError> for LumatoneMidiError {
  fn from(e: PortInfoError) -> Self {
    LumatoneMidiError::MidiPortInfoError(e)
  }
}

impl From<ConnectError<MidiInput>> for LumatoneMidiError {
  fn from(e: ConnectError<MidiInput>) -> Self {
    LumatoneMidiError::MidiInputConnectError(e)
  }
}

impl From<ConnectError<MidiOutput>> for LumatoneMidiError {
  fn from(e: ConnectError<MidiOutput>) -> Self {
    LumatoneMidiError::MidiOutputConnectError(e)
  }
}

impl Display for LumatoneMidiError {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    use LumatoneMidiError::*;
    match self {
      // InvalidCommandInput(cmd_id, msg) => {
      //   write!(f, "invalid command input for {:?}: {}", cmd_id, msg)
      // }
      NotLumatoneMessage(msg) => write!(f, "message is not a lumatone message: {:?}", msg),

      MessageTooShort { expected, actual } => write!(
        f,
        "expected message to have length of at least {expected}, but received {actual}"
      ),

      MessagePayloadTooShort { expected, actual } => write!(
        f,
        "expected message payload to have length of {expected}, but received {actual}"
      ),

      UnknownCommandId(id) => write!(f, "unknown command id {:x}", id),

      UnexpectedCommandId { expected, actual } => write!(
        f,
        "unexpected command id in incoming message. expected {:?}, received {:?}",
        expected, actual
      ),

      InvalidResponseMessage(msg) => write!(f, "received invalid response: {msg}"),

      MidiPortNotFound(name) => write!(f, "unable to find midi port with name: {name}"),

      MidiPortInfoError(err) => write!(f, "error getting midi port info: {err}"),

      MidiInitError(err) => write!(f, "midi init error: {err}"),

      MidiSendError(err) => write!(f, "midi send error: {err}"),

      MidiInputConnectError(err) => write!(f, "midi input connection error: {err}"),

      MidiOutputConnectError(err) => write!(f, "midi output connection error: {err}"),

      InvalidStateTransition(msg) => write!(f, "invalid state transition: {msg}"),

      DeviceDetectionFailed(msg) => write!(f, "device detection failed: {msg}"),

      InvalidBoardIndex(n) => write!(f, "invalid board index: {n}"),

      UnsupportedCommandId(cmd_id, context) => {
        write!(f, "unsupported command id: {cmd_id:?}: {context}")
      }

      InvalidMidiChannel(n) => write!(f, "invalid midi channel {n}. Valid range is 1 ..= 16"),

      InvalidLumatoneKeyIndex(n) => {
        write!(f, "invalid lumatone key index {n}. Valid range is 0 ..= 55")
      }

      InvalidPresetIndex(n) => write!(f, "invalid preset index {n}. Valid range is 0 ..= 9"),
    }
  }
}
