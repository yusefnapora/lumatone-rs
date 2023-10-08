use super::constants::CommandId;
use serde::{Serialize, Deserialize};

use error_stack::Context;
use std::fmt::Display;

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq)]
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
  MessagePayloadInvalid(String),
  UnknownCommandId(u8),
  UnexpectedCommandId {
    expected: CommandId,
    actual: CommandId,
  },
  UnsupportedCommandId(CommandId, String),
  InvalidResponseMessage(String),

  InvalidStateTransition(String),
  DeviceDetectionFailed,
  DeviceConnectionError,
  DeviceSendError,

  ResponseDecodingError,

  InvalidBoardIndex(u8),
  InvalidMidiChannel(u8),
  InvalidLumatoneKeyIndex(u8),
  InvalidPresetIndex(u8),
}

impl Context for LumatoneMidiError {}

impl Display for LumatoneMidiError {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    use LumatoneMidiError::*;
    match self {
      NotLumatoneMessage(msg) => write!(f, "message is not a lumatone message: {:?}", msg),

      MessageTooShort { expected, actual } => write!(
        f,
        "expected message to have length of at least {expected}, but received {actual}"
      ),

      MessagePayloadTooShort { expected, actual } => write!(
        f,
        "expected message payload to have length of {expected}, but received {actual}"
      ),

      MessagePayloadInvalid(msg) => write!(f, "invalid message payload: {msg}"),

      UnknownCommandId(id) => write!(f, "unknown command id {:x}", id),

      UnexpectedCommandId { expected, actual } => write!(
        f,
        "unexpected command id in incoming message. expected {:?}, received {:?}",
        expected, actual
      ),

      InvalidResponseMessage(msg) => write!(f, "received invalid response: {msg}"),

      InvalidStateTransition(msg) => write!(f, "invalid state transition: {msg}"),

      DeviceDetectionFailed => write!(f, "device detection failed"),

      DeviceConnectionError => write!(f, "failed to connect to device"),

      DeviceSendError => write!(f, "failed to send message to device"),

      ResponseDecodingError => write!(f, "failed to decode response from device"),

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
