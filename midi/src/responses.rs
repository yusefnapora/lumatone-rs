#![allow(unused)]
use std::fmt::Display;

use super::{
  constants::{BoardIndex, CommandId, MidiChannel, TEST_ECHO},
  error::LumatoneMidiError,
  sysex::{
    is_lumatone_message, message_command_id, message_payload, strip_sysex_markers, SysexTable,
    VelocityIntervalTable, BOARD_IND,
  },
};

#[derive(Debug)]
pub enum Response {
  /// indicates that the command was successful, but no additional data was returned.
  Ack(CommandId),

  Pong(u32),

  /// 8-bit key data for red LED intensity. 112 bytes, lower and upper nibbles for 56 values
  RedLEDConfig(BoardIndex, Vec<u8>),

  /// 8-bit key data for green LED intensity. 112 bytes, lower and upper nibbles for 56 values
  GreenLEDConfig(BoardIndex, Vec<u8>),

  /// 8-bit key data for blue LED intensity. 112 bytes, lower and upper nibbles for 56 values
  BlueLEDConfig(BoardIndex, Vec<u8>),

  /// channel data for note configuration. 55 or 56 bytes
  ChannelConfig(BoardIndex, Vec<MidiChannel>),

  /// 7-bit key data for note configuration. 55 or 56 bytes
  NoteConfig(BoardIndex, Vec<u8>),

  /// 7-bit key type data for key configuration. 55 or 56 bytes
  KeyTypeConfig(BoardIndex, Vec<u8>),

  /// 8-bit key data for maximums of adc threshold. 55 or 56 bytes
  KeyMaxThresholds(BoardIndex, Vec<u8>),

  /// 8-bit key data for minimums of adc threshold. 55 or 56 bytes
  KeyMinThresholds(BoardIndex, Vec<u8>),

  /// 8-bit key data for maximums of adc threshold for aftertouch triggering. 55 or 56 bytes
  AftertouchMaxThresholds(BoardIndex, Vec<u8>),

  /// key validity data for board, whether or not each key meets threshold specs
  KeyValidity(BoardIndex, Vec<bool>),

  /// 7-bit fader type configuration of board, 56 bytes
  FaderTypeConfig(BoardIndex, Vec<u8>),

  /// 7-bit velocity configuration of keyboard, 128 bytes
  OnOffVelocityConfig(Box<SysexTable>),

  /// 7-bit fader configuration of keyboard, 128 bytes
  FaderConfig(Box<SysexTable>),

  /// 7-bit aftertouch configuration of keyboard, 128 bytes
  AftertouchConfig(Box<SysexTable>),

  /// 7-bit lumatouch configuration of keyboard, 128 bytes
  LumatouchConfig(Box<SysexTable>),

  /// 12-bit velocity interval configuration of keyboard; 127 values
  VelocityIntervalConfig(Box<VelocityIntervalTable>),

  /// Serial ID of keyboard (6 bytes).
  SerialId([u8; 6]),

  /// Firmware version number
  FirmwareRevision {
    major: u8,
    minor: u8,
    revision: u8,
  },

  /// All threshold values for a given board
  BoardThresholds {
    board_index: BoardIndex,
    min_high: u8,
    min_low: u8,
    max: u8,
    aftertouch: u8,
    cc: u8,
  },

  /// The continuous controller and aftertouch sensitivity values for a given board
  BoardSensitivity {
    board_index: BoardIndex,
    cc: u8,
    aftertouch: u8,
  },

  /// The MIDI channel numbers for all peripherals
  PeripheralChannels {
    pitch_wheel: MidiChannel,
    mod_wheel: MidiChannel,
    expression: MidiChannel,
    sustain: MidiChannel,
  },

  /// 12-bit expression pedal calibration status values, automatically sent every 100ms when in expression calibration mode
  ExpressionCalibrationStatus {
    min_bound: u16,
    max_bound: u16,
    valid: bool,
  },

  /// 12-bit pitch & mod wheel calibration status values, automatically sent every 100ms when in pitch/mod calibration mode
  WheelCalibrationStatus {
    center_pitch: u16,
    min_pitch: u16,
    max_pitch: u16,
    min_mod: u16,
    max_mod: u16,
  },

  /// Aftertouch trigger delay value for a given board
  AftertouchTriggerDelay(BoardIndex, u8),

  /// 12-bit Lumatouch note off delay of a certain board
  LumatouchNoteOffDelay(BoardIndex, u16),

  /// 12-bit expression pedal adc threshold, a 12-bit value
  ExpressionPedalThreshold(u16),
}

impl Response {
  pub fn from_sysex_message(msg: &[u8]) -> Result<Response, LumatoneMidiError> {
    use CommandId::*;
    let cmd_id = message_command_id(msg)?;
    match cmd_id {
      LumaPing => decode_ping(msg).map(|val| Response::Pong(val)),

      GetRedLedConfig => unpack_octave_data_8bit(msg).map(|(b, d)| Response::RedLEDConfig(b, d)),

      GetGreenLedConfig => {
        unpack_octave_data_8bit(msg).map(|(b, d)| Response::GreenLEDConfig(b, d))
      }

      GetBlueLedConfig => unpack_octave_data_8bit(msg).map(|(b, d)| Response::BlueLEDConfig(b, d)),

      GetChannelConfig => unpack_channel_config(msg),

      GetNoteConfig => unpack_octave_data_7bit(msg).map(|(b, d)| Response::NoteConfig(b, d)),

      GetKeytypeConfig => unpack_octave_data_7bit(msg).map(|(b, d)| Response::KeyTypeConfig(b, d)),

      GetMaxThreshold => {
        unpack_octave_data_8bit(msg).map(|(b, d)| Response::KeyMaxThresholds(b, d))
      }

      GetMinThreshold => {
        unpack_octave_data_8bit(msg).map(|(b, d)| Response::KeyMinThresholds(b, d))
      }

      GetAftertouchMax => {
        unpack_octave_data_8bit(msg).map(|(b, d)| Response::AftertouchMaxThresholds(b, d))
      }

      GetKeyValidity => unpack_key_validity(msg),

      GetVelocityConfig => unpack_sysex_config_table(msg).map(Response::OnOffVelocityConfig),

      GetFaderConfig => unpack_sysex_config_table(msg).map(Response::FaderConfig),

      GetAftertouchConfig => unpack_sysex_config_table(msg).map(Response::AftertouchConfig),

      GetLumatouchConfig => unpack_sysex_config_table(msg).map(Response::LumatouchConfig),

      GetVelocityIntervals => unpack_velocity_intervals(msg),

      GetSerialIdentity => unpack_serial_id(msg),

      GetFirmwareRevision => unpack_firmware_revision(msg),

      GetBoardThresholdValues => unpack_board_thresholds(msg),

      GetBoardSensitivityValues => unpack_board_sensitivity(msg),

      GetPeripheralChannels => unpack_peripheral_channels(msg),

      CalibrateExpressionPedal => unpack_expression_calibration_status(msg),

      CalibratePitchModWheel => unpack_wheel_calibration_status(msg),

      GetAftertouchTriggerDelay => unpack_aftertouch_trigger_delay(msg),

      GetLumatouchNoteOffDelay => unpack_lumatouch_on_off_delay(msg),

      GetExpressionPedalThreshold => unpack_expression_threshold(msg),

      _ => Ok(Response::Ack(cmd_id)),
    }
  }
}

impl Display for Response {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    use Response::*;
    match self {
      Ack(cmd_id) => write!(f, "Ack({cmd_id:?})"),
      Pong(val) => write!(f, "Pong({val})"),
      RedLEDConfig(board, _) => write!(f, "RedLEDConfig({board}, <table...>)"),
      GreenLEDConfig(board, _) => write!(f, "GreenLEDConfig({board}, <table..>)"),
      BlueLEDConfig(board, _) => write!(f, "BlueLEDConfig({board}, <table..>)"),
      ChannelConfig(board, _) => write!(f, "ChannelConfig({board}, <table..>)"),
      NoteConfig(board, _) => write!(f, "NoteConfig({board}, <table..>)"),
      KeyTypeConfig(board, _) => write!(f, "KeyTypeConfig({board}, <table..>)"),
      KeyMaxThresholds(board, _) => write!(f, "KeyMaxThresholds({board}, <table..>)"),
      KeyMinThresholds(board, _) => write!(f, "KeyMinThresholds({board}, <table..>)"),
      AftertouchMaxThresholds(board, _) => write!(f, "AftertouchMaxThresholds({board}, <table..>)"),
      KeyValidity(board, _) => write!(f, "KeyValidity({board}, <table..>)"),
      FaderTypeConfig(board, _) => write!(f, "FaderTypeConfig({board}, <table..>)"),
      OnOffVelocityConfig(_) => write!(f, "OnOffVelocityConfig(<table...>)"),
      FaderConfig(_) => write!(f, "FaderConfig(<table...>)"),
      AftertouchConfig(_) => write!(f, "AftertouchConfig(<table...>)"),
      LumatouchConfig(_) => write!(f, "LumatouchConfig(<table...>)"),
      VelocityIntervalConfig(_) => write!(f, "VelocityIntervalConfig(<table...>)"),
      SerialId(id) => write!(f, "SerialId({id:?})"),
      FirmwareRevision {
        major,
        minor,
        revision,
      } => write!(f, "FirmwareRevision(\"{major}.{minor}.{revision}\")"),
      BoardThresholds {
        board_index,
        min_high,
        min_low,
        max,
        aftertouch,
        cc,
      } => todo!(),
      BoardSensitivity {
        board_index,
        cc,
        aftertouch,
      } => todo!(),
      PeripheralChannels {
        pitch_wheel,
        mod_wheel,
        expression,
        sustain,
      } => todo!(),
      ExpressionCalibrationStatus {
        min_bound,
        max_bound,
        valid,
      } => todo!(),
      WheelCalibrationStatus {
        center_pitch,
        min_pitch,
        max_pitch,
        min_mod,
        max_mod,
      } => todo!(),
      AftertouchTriggerDelay(board, val) => write!(f, "AftertouchTriggerDelay({board}, {val})"),
      LumatouchNoteOffDelay(board, val) => write!(f, "LumatouchNoteOffDelay({board}, {val})"),
      ExpressionPedalThreshold(val) => write!(f, "ExpressionPedalThreshold({val})"),
    }
  }
}

fn message_board_index(msg: &[u8]) -> Result<BoardIndex, LumatoneMidiError> {
    if msg.len() <= BOARD_IND {
			return Err(LumatoneMidiError::MessageTooShort {
				expected: BOARD_IND + 1,
				actual: msg.len()
			});
		}

  BoardIndex::try_from(msg[BOARD_IND])
}

// region: Sysex Decoders

/// Attempts to decode a sysex message as a "ping" response,
/// returning the encoded payload value on success.
pub fn decode_ping(msg: &[u8]) -> Result<u32, LumatoneMidiError> {
  if !is_lumatone_message(msg) {
    return Err(LumatoneMidiError::NotLumatoneMessage(msg.to_vec()));
  }

  let cmd_id = message_command_id(msg)?;
  if cmd_id != CommandId::LumaPing {
    return Err(LumatoneMidiError::UnexpectedCommandId {
      expected: CommandId::LumaPing,
      actual: cmd_id,
    });
  }

  let payload = message_payload(msg)?;
  if payload.len() < 4 {
    return Err(LumatoneMidiError::MessagePayloadTooShort {
      expected: 4,
      actual: payload.len(),
    });
  }

  if payload[0] != TEST_ECHO {
    return Err(LumatoneMidiError::InvalidResponseMessage(
      "ping response has invalid echo flag value".to_string(),
    ));
  }

  let value: u32 = ((payload[1] as u32) << 14) | ((payload[2] as u32) << 7) | (payload[3] as u32);
  Ok(value)
}

// endregion

// region: data unpacking helper fns
fn valid_lumatone_msg<'a>(msg: &'a [u8]) -> Result<&'a [u8], LumatoneMidiError> {
  let msg = strip_sysex_markers(msg);
  if !is_lumatone_message(msg) {
    Err(LumatoneMidiError::NotLumatoneMessage(msg.to_vec()))
  } else {
    Ok(msg)
  }
}

fn payload_with_len<'a>(msg: &'a [u8], len: usize) -> Result<&'a [u8], LumatoneMidiError> {
  let msg = valid_lumatone_msg(msg)?;

  let payload = message_payload(msg)?;
  if payload.len() < len {
    Err(LumatoneMidiError::MessagePayloadTooShort {
      expected: len,
      actual: payload.len(),
    })
  } else {
    Ok(&payload[0..len])
  }
}

fn unpack_sysex_config_table(msg: &[u8]) -> Result<Box<SysexTable>, LumatoneMidiError> {
  let payload = payload_with_len(msg, 128)?;
  let table: SysexTable = payload.try_into().unwrap();
  Ok(Box::new(table))
}

fn unpack_octave_data_8bit(msg: &[u8]) -> Result<(BoardIndex, Vec<u8>), LumatoneMidiError> {
  let msg = valid_lumatone_msg(msg)?;
  let board_index = message_board_index(msg)?;
  let payload = message_payload(msg)?;
  Ok((board_index, unpack_8bit(payload)))
}

fn unpack_octave_data_7bit(msg: &[u8]) -> Result<(BoardIndex, Vec<u8>), LumatoneMidiError> {
  let msg = valid_lumatone_msg(msg)?;
  let board_index = message_board_index(msg)?;
  let payload = message_payload(msg)?;
  Ok((board_index, payload.to_vec()))
}

fn unpack_channel_config(msg: &[u8]) -> Result<Response, LumatoneMidiError> {
  let msg = valid_lumatone_msg(msg)?;
  let board_index = message_board_index(msg)?;
  let payload = message_payload(msg)?;
  let mut channels = Vec::with_capacity(payload.len());
  for byte in payload {
    let ch = MidiChannel::try_from_zero_indexed(*byte)?;
    channels.push(ch);
  }
  let response = Response::ChannelConfig(board_index, channels);
  Ok(response)
}

fn unpack_key_validity(msg: &[u8]) -> Result<Response, LumatoneMidiError> {
  let msg = valid_lumatone_msg(msg)?;
  let board_index = message_board_index(msg)?;
  let payload = message_payload(msg)?;
  let bools = payload.iter().map(|n| *n != 0).collect();
  Ok(Response::KeyValidity(board_index, bools))
}

fn unpack_velocity_intervals(msg: &[u8]) -> Result<Response, LumatoneMidiError> {
  let payload = payload_with_len(msg, 254)?;
  let data = unpack_12bit_from_7bit(payload);
  let table: VelocityIntervalTable = data.try_into().unwrap();
  Ok(Response::VelocityIntervalConfig(Box::new(table)))
}

fn unpack_serial_id(msg: &[u8]) -> Result<Response, LumatoneMidiError> {
  // TODO: the C++ driver has a check for msg[MSG_STATUS] == TEST_ECHO
  // add that if it seems necessary.

  // Also note that we're not handling early firmware versions that respond with an ACK but no serial number.

  let payload = payload_with_len(msg, 6)?;
  let serial: [u8; 6] = payload.try_into().unwrap();
  Ok(Response::SerialId(serial))
}

fn unpack_firmware_revision(msg: &[u8]) -> Result<Response, LumatoneMidiError> {
  let payload = payload_with_len(msg, 3)?;
  Ok(Response::FirmwareRevision {
    major: payload[0],
    minor: payload[1],
    revision: payload[2],
  })
}

fn unpack_board_thresholds(msg: &[u8]) -> Result<Response, LumatoneMidiError> {
  let payload = payload_with_len(msg, 10)?;
  let (board_index, data) = unpack_octave_data_8bit(payload)?;
  Ok(Response::BoardThresholds {
    board_index,
    min_high: data[0],
    min_low: data[1],
    max: data[2],
    aftertouch: data[3],
    cc: data[4],
  })
}

fn unpack_board_sensitivity(msg: &[u8]) -> Result<Response, LumatoneMidiError> {
  let payload = payload_with_len(msg, 4)?;
  let (board_index, data) = unpack_octave_data_8bit(msg)?;
  Ok(Response::BoardSensitivity {
    board_index,
    cc: data[0],
    aftertouch: data[1],
  })
}

fn unpack_peripheral_channels(msg: &[u8]) -> Result<Response, LumatoneMidiError> {
  let payload = payload_with_len(msg, 4)?;
  let (board_index, data) = unpack_octave_data_7bit(msg)?;

  let pitch_wheel = MidiChannel::try_from_zero_indexed(data[0])?;
  let mod_wheel = MidiChannel::try_from_zero_indexed(data[1])?;
  let expression = MidiChannel::try_from_zero_indexed(data[2])?;
  let sustain = MidiChannel::try_from_zero_indexed(data[3])?;

  Ok(Response::PeripheralChannels {
    pitch_wheel,
    mod_wheel,
    expression,
    sustain,
  })
}

fn unpack_expression_calibration_status(msg: &[u8]) -> Result<Response, LumatoneMidiError> {
  let payload = payload_with_len(msg, 15)?;

  // the min and max bounds are encoded into the first six bytes of the payload
  let bounds_data = unpack_12bit_from_4bit(payload);
  let min_bound = bounds_data[0];
  let max_bound = bounds_data[1];

  // The C++ version looks incorrect to me... they have:
  // ```
  // valid = response.getSysExData()[PAYLOAD_INIT + 3];
  // ```
  // but the max bound is at [PAYLOAD_INIT + 3], since each 12bit value takes 3 bytes.
  // I'm going to assume this is supposed to be PAYLOAD_INIT + 6
  let valid = payload[6] != 0;
  Ok(Response::ExpressionCalibrationStatus {
    min_bound,
    max_bound,
    valid,
  })
}

fn unpack_wheel_calibration_status(msg: &[u8]) -> Result<Response, LumatoneMidiError> {
  let payload = payload_with_len(msg, 15)?;
  let data = unpack_12bit_from_4bit(payload);
  let center_pitch = data[0];
  let min_pitch = data[1];
  let max_pitch = data[2];
  let min_mod = data[3];
  let max_mod = data[4];
  Ok(Response::WheelCalibrationStatus {
    center_pitch,
    min_pitch,
    max_pitch,
    min_mod,
    max_mod,
  })
}

fn unpack_aftertouch_trigger_delay(msg: &[u8]) -> Result<Response, LumatoneMidiError> {
  let payload = payload_with_len(msg, 2)?;
  let (board_index, data) = unpack_octave_data_8bit(payload)?;
  Ok(Response::AftertouchTriggerDelay(board_index, data[0]))
}

fn unpack_lumatouch_on_off_delay(msg: &[u8]) -> Result<Response, LumatoneMidiError> {
  let payload = payload_with_len(msg, 3)?;
  let board_index = message_board_index(msg)?;
  let data = unpack_12bit_from_4bit(payload);
  let delay = data[0];
  Ok(Response::LumatouchNoteOffDelay(board_index, delay))
}

fn unpack_expression_threshold(msg: &[u8]) -> Result<Response, LumatoneMidiError> {
  let payload = payload_with_len(msg, 3)?;
  let data = unpack_12bit_from_4bit(payload);
  let threshold = data[0];
  Ok(Response::ExpressionPedalThreshold(threshold))
}

/// Generic unpacking of 8-bit data from a SysEx message payload
fn unpack_8bit(payload: &[u8]) -> Vec<u8> {
  payload
    .chunks_exact(2)
    .flat_map(|c| vec![c[0] << 4, c[1]])
    .collect()
}

/// Generic unpacking of 12-bit data from a SysEx message, when packed with two 7-bit values
fn unpack_12bit_from_7bit(payload: &[u8]) -> Vec<u16> {
  payload
    .chunks_exact(2)
    .map(|c| ((c[0] as u16) << 6) | (c[1] as u16))
    .collect()
}

/// Generic unpacking of 12-bit data from a SysEx message, when packed with three 4-bit values
fn unpack_12bit_from_4bit(payload: &[u8]) -> Vec<u16> {
  payload
    .chunks_exact(3)
    .map(|c| ((c[0] as u16) << 8) | ((c[1] as u16) << 4) | (c[2] as u16))
    .collect()
}

// endregion
