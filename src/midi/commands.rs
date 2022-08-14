use super::{constants::{BoardIndex, CommandId as CMD, TEST_ECHO}, sysex::{EncodedSysex, create_sysex, create_extended_key_color_sysex}};

use std::error::Error;

/// CMD 0x0: Send a single key's functional configuration
pub fn set_key_function_parameters(
  board_index: BoardIndex,
  key_index: u8,
  note_or_cc_num: u8,
  midi_channel: u8,
  key_type: u8,
  fader_up_is_null: bool
) -> EncodedSysex {
  let channel = (midi_channel - 1) & 0xf;
  let type_byte: u8 = if fader_up_is_null {
    (1 << 4) | key_type
  } else {
    key_type
  };
  create_sysex(board_index, CMD::CHANGE_KEY_NOTE, vec![
    key_index,
    note_or_cc_num,
    channel,
    type_byte
  ])
}

/// CMD 0x01: Send a single key's LED channel intensities
pub fn set_key_light_parameters(
  board_index: BoardIndex,
  key_index: u8,
  red: u8,
  green: u8,
  blue: u8
) -> EncodedSysex {
  create_extended_key_color_sysex(board_index, CMD::SET_KEY_COLOUR, key_index, red, green, blue)
}

/// CMD 0x02: Save current configuration to a specified preset button index
pub fn save_program(preset_number: u8) -> Result<EncodedSysex, Box<dyn Error>> {
  if preset_number > 9 {
    return Err("invalid input: max preset number is 9".into());
  }

  Ok(
    create_sysex(board_index, CMD::SAVE_PROGRAM, vec![preset_number])
  )
}

pub fn ping(value: u32) -> EncodedSysex {
  let val = value & 0xfffffff; // limit to 28 bits
  create_sysex(BoardIndex::SERVER, CMD::LUMA_PING, vec![
    TEST_ECHO,
    (val >> 14) & 0x7f,
    (val >> 7) & 0x7f,
    val & 0x7f
  ])
}

// TODO: add remaining commands
