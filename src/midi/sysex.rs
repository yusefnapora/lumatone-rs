// TODO: 
// - [ ] structs for lumatone commands
// - [ ] encoder to convert commands to/from sysex byte stream

use super::constants::{BoardIndex, CommandId, MANUFACTURER_ID};

// index into sysex data of various fields
const INDEX_MANU_0: u8 = 0x0;
const INDEX_MANU_1: u8 = 0x1;
const INDEX_MANU_3: u8 = 0x2;
const INDEX_BOARD_IND: u8 = 0x3;
const INDEX_CMD_ID: u8 = 0x4;
const INDEX_MSG_STATUS: u8 = 0x5;
const INDEX_CALIB_MODE: u8 = 0x5;
const INDEX_PAYLOAD_INIT: u8 = 0x6;

const ECHO_FLAG: u8 = 0x5; // used to differentiate test responses from MIDI 
const TEST_ECHO: u8 = 0x7f; // should not be returned by lumatone

pub type EncodedSysex = Vec<u8>;

pub fn create_sysex(board_index: BoardIndex, cmd: CommandId, data: Vec<u8>) -> EncodedSysex {
  // TODO: concat board_index, cmd, data and return
}

pub fn create_sysex_toggle(board_index: BoardIndex, cmd: CommandId, state: bool) -> EncodedSysex {
  let s: u8 = if state { 1 } else { 0 };
  create_sysex(board_index, cmd, vec![s])
}

pub fn create_extended_key_color_sysex(
  board_index: BoardIndex,
  cmd: CommandId,
  key_index: u8,
  red: u8,
  green: u8,
  blue: u8
) -> EncodedSysex {
  let mut colors = encode_rgb(red, green, blue);
  let mut data = vec![key_index];
  data.append(&mut colors);
  create_sysex(board_index, cmd, data)
}

pub fn create_extended_macro_color_sysex(
  cmd: CommandId,
  red: u8,
  green: u8,
  blue: u8
) -> EncodedSysex {
  let colors = encode_rgb(red, green, blue);
  create_sysex(BoardIndex::SERVER, cmd, colors)
}

/**
 * Returns the given RGB values, encoded into 6 u8's with the lower 4 bits set.
 */
fn encode_rgb(red: u8, green: u8, blue: u8) -> Vec<u8> {
  let red_hi = red >> 4;
  let red_lo = red & 0xf;
  let green_hi = green >> 4;
  let green_lo = green & 0xf;
  let blue_hi = blue >> 4;
  let blue_lo = blue & 0xf;
  vec![ red_hi, red_lo, green_hi, green_lo, blue_hi, blue_lo ]
}

impl EncodedSysex {
  pub fn is_lumatone_message(&self) -> bool {
    if self.len() < 3 {
      return false
    }
    for (a, b) in MANUFACTURER_ID.iter().zip(self.iter()) {
      if *a != *b {
        return false
      }
    }
    return true
  }
}