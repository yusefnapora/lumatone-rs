#![allow(dead_code)]

use super::{
  constants::{BoardIndex, CommandId, RGBColor, ResponseStatusCode, MANUFACTURER_ID},
  error::LumatoneMidiError,
};
use num_traits::FromPrimitive;

// index into sysex data of various fields
pub const MANU_0: usize = 0x0;
pub const MANU_1: usize = 0x1;
pub const MANU_3: usize = 0x2;
pub const BOARD_IND: usize = 0x3;
pub const CMD_ID: usize = 0x4;
pub const MSG_STATUS: usize = 0x5;
pub const CALIB_MODE: usize = 0x5;
pub const PAYLOAD_INIT: usize = 0x6;

pub const SYSEX_START: u8 = 0xf0;
pub const SYSEX_END: u8 = 0xf7;

pub type EncodedSysex = Vec<u8>;

/// Some commands send "tables" of config data (e.g. key velocity, etc).
/// Tables are always 128 elements long.
pub type SysexTable = Vec<u8>;

/// The velocity interval table contains 127 12-bit values.
pub type VelocityIntervalTable = Vec<u16>;

pub fn reverse_table(t: &SysexTable) -> SysexTable {
  let mut r = t.clone();
  r.reverse();
  r
}

pub fn to_hex_debug_str(msg: &[u8]) -> String {
  let s = msg
    .iter()
    .map(|b| format!("{b:x}"))
    .collect::<Vec<String>>()
    .join(" ");
  format!("[ {s} ]")
}

pub fn create_sysex(board_index: BoardIndex, cmd: CommandId, data: Vec<u8>) -> EncodedSysex {
  let mut sysex: Vec<u8> = vec![SYSEX_START];
  sysex.extend(MANUFACTURER_ID.iter());
  sysex.push(board_index.into());
  sysex.push(cmd.into());
  sysex.extend(data.iter());

  // The C++ driver seems to always send a minimum of 9 bytes, not counting the SYSEX_START marker
  // So we add a little padding if we're sending less than 9 bytes.
  if sysex.len() < 10 {
    let pad = 10 - sysex.len();
    for _ in 0..pad {
      sysex.push(0);
    }
  }
  sysex.push(SYSEX_END);
  sysex
}

pub fn create_sysex_toggle(board_index: BoardIndex, cmd: CommandId, state: bool) -> EncodedSysex {
  let s: u8 = if state { 1 } else { 0 };
  create_sysex(board_index, cmd, vec![s])
}

pub fn create_zero_arg_sysex(board_index: BoardIndex, cmd: CommandId) -> EncodedSysex {
  create_sysex(board_index, cmd, vec![])
}

pub fn create_zero_arg_server_sysex(cmd: CommandId) -> EncodedSysex {
  create_sysex(BoardIndex::Server, cmd, vec![])
}

pub fn create_single_arg_server_sysex(cmd: CommandId, value: u8) -> EncodedSysex {
  create_sysex(BoardIndex::Server, cmd, vec![value])
}

pub fn create_extended_key_color_sysex(
  board_index: BoardIndex,
  cmd: CommandId,
  key_index: u8,
  color: &RGBColor,
) -> EncodedSysex {
  let mut data = vec![key_index];
  data.extend(color.to_bytes());
  create_sysex(board_index, cmd, data)
}

pub fn create_extended_macro_color_sysex(cmd: CommandId, color: &RGBColor) -> EncodedSysex {
  create_sysex(BoardIndex::Server, cmd, color.to_bytes())
}

pub fn create_table_sysex(cmd: CommandId, table: &SysexTable) -> EncodedSysex {
  create_sysex(BoardIndex::Server, cmd, table.to_vec())
}

pub fn strip_sysex_markers<'a>(msg: &'a [u8]) -> &'a [u8] {
  if msg.len() == 0 {
    return &msg;
  }

  let start = if msg[0] == SYSEX_START { 1 } else { 0 };
  let mut end = msg.len() - 1;
  if msg[end] == SYSEX_END {
    end -= 1;
  }
  &msg[start..=end]
}

pub fn is_lumatone_message(msg: &[u8]) -> bool {
  let msg = strip_sysex_markers(msg);

  if msg.len() < 3 {
    return false;
  }
  for (a, b) in MANUFACTURER_ID.iter().zip(msg.iter()) {
    if *a != *b {
      return false;
    }
  }
  return true;
}

pub fn message_payload<'a>(msg: &'a [u8]) -> Result<&'a [u8], LumatoneMidiError> {
  let msg = strip_sysex_markers(msg);
  if msg.len() <= PAYLOAD_INIT {
    return Err(LumatoneMidiError::MessageTooShort {
      expected: PAYLOAD_INIT + 1,
      actual: msg.len(),
    });
  }
  Ok(&msg[PAYLOAD_INIT..])
}

pub fn message_command_id(msg: &[u8]) -> Result<CommandId, LumatoneMidiError> {
  let msg = strip_sysex_markers(msg);
  if msg.len() <= CMD_ID {
    return Err(LumatoneMidiError::MessageTooShort {
      expected: CMD_ID + 1,
      actual: msg.len(),
    });
  }
  let cmd_id = msg[CMD_ID];
  let cmd: Option<CommandId> = FromPrimitive::from_u8(cmd_id);
  cmd.ok_or(LumatoneMidiError::UnknownCommandId(cmd_id))
}

pub fn message_answer_code(msg: &[u8]) -> ResponseStatusCode {
  let msg = strip_sysex_markers(msg);
  if msg.len() <= MSG_STATUS {
    return ResponseStatusCode::Unknown;
  }

  let status_byte = msg[MSG_STATUS];
  let status: Option<ResponseStatusCode> = FromPrimitive::from_u8(status_byte);
  status.unwrap_or(ResponseStatusCode::Unknown)
}

pub fn is_response_to_message(outgoing: &[u8], incoming: &[u8]) -> bool {
  let outgoing = strip_sysex_markers(outgoing);
  let incoming = strip_sysex_markers(incoming);

  if !is_lumatone_message(incoming) {
    return false;
  }

  if incoming.len() <= CMD_ID || outgoing.len() < CMD_ID {
    return false;
  }

  incoming[CMD_ID] == outgoing[CMD_ID] && incoming[BOARD_IND] == outgoing[BOARD_IND]
}
