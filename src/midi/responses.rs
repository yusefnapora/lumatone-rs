use super::{
  constants::{BoardIndex, CommandId, MidiChannel, TEST_ECHO},
  error::LumatoneMidiError,
  sysex::{
    is_lumatone_message, message_command_id, message_payload, strip_sysex_markers, SysexTable,
    VelocityIntervalTable, BOARD_IND,
  },
};

pub enum Response {
  Ping(u32),

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

  /// The MIDI channel numbers for all peripherals
  PeripheralChannels {
    pitch_wheel: MidiChannel,
    mod_wheel: MidiChannel,
    expression: MidiChannel,
    sustain: MidiChannel,
  },

  /// 12-bit expression pedal calibration status values, automatically sent every 100ms when in expression calibration mode
  ExpressionCalibrationStatus {
    min_bound: u8,
    max_bound: u8,
    valid: bool,
  },

  /// 12-bit pitch & mod wheel calibration status values, automatically sent every 100ms when in pitch/mod calibration
  PitchModCalibrationStatus {
    center_pitch: u8,
    min_pitch: u8,
    max_pitch: u8,
    min_mod: u8,
    max_mod: u8,
  },

  /// Aftertouch trigger delay value for a given board
  AftertouchTriggerDelayResponse(BoardIndex, u8),

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
      LumaPing => decode_ping(msg).map(|val| Response::Ping(val)),

      GetRedLedConfig => unpack_octave_data_8bit(msg).map(|(b, d)| Response::RedLEDConfig(b, d)),

      GetGreenLedConfig => {
        unpack_octave_data_8bit(msg).map(|(b, d)| Response::GreenLEDConfig(b, d))
      }

      GetBlueLedConfig => unpack_octave_data_8bit(msg).map(|(b, d)| Response::BlueLEDConfig(b, d)),

      GetChannelConfig => unpack_channel_config_response(msg),

      GetNoteConfig => unpack_octave_data_7bit(msg).map(|(b, d)| Response::NoteConfig(b, d)),

      GetKeytypeConfig => unpack_octave_data_7bit(msg).map(|(b, d)| Response::KeyTypeConfig(b, d)),

      GetMaxThreshold => unpack_octave_data_8bit(msg).map(|(b, d)| Response::KeyMaxThresholds(b, d)),

      GetMinThreshold => unpack_octave_data_8bit(msg).map(|(b, d)| Response::KeyMinThresholds(b, d)),

      GetAftertouchMax => unpack_octave_data_8bit(msg).map(|(b, d)| Response::AftertouchMaxThresholds(b, d)),

      GetKeyValidity => unpack_key_validity_response(msg),

      GetVelocityConfig => unpack_sysex_config_table(msg).map( Response::OnOffVelocityConfig),

      GetFaderConfig => unpack_sysex_config_table(msg).map(Response::FaderConfig),

      GetAftertouchConfig => unpack_sysex_config_table(msg).map(Response::AftertouchConfig),

      GetLumatouchConfig => unpack_sysex_config_table(msg).map(Response::LumatouchConfig),

      GetVelocityIntervals => todo!(),

      GetSerialIdentity => todo!(),

      GetFirmwareRevision => todo!(),

      GetBoardThresholdValues => todo!(),

      GetBoardSensitivityValues => todo!(),

      GetPeripheralChannels => todo!(),

      CalibrateExpressionPedal => todo!(),

      CalibratePitchModWheel => todo!(),

      GetAftertouchTriggerDelay => todo!(),

      GetLumatouchNoteOffDelay => todo!(),

      GetExpressionPedalThreshold => todo!(),
      
      _ => Err(LumatoneMidiError::UnsupportedCommandId(
        cmd_id,
        "no response decoder".to_string(),
      )),
    }
  }
}

fn message_board_index(msg: &[u8]) -> Result<BoardIndex, LumatoneMidiError> {
  if msg.len() <= BOARD_IND {
    return Err(LumatoneMidiError::MessageTooShort {
      expected: BOARD_IND + 1,
      actual: msg.len(),
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

fn unpack_sysex_config_table(msg: &[u8]) -> Result<Box<SysexTable>, LumatoneMidiError> {
  let payload = message_payload(msg)?;
  if payload.len() < 128 {
    return Err(LumatoneMidiError::MessagePayloadTooShort { expected: 128, actual: payload.len() });
  }

  let mut table = [0; 128];
  for (i, n) in payload.iter().enumerate() {
    table[i] = *n;
  }
  Ok(Box::new(table))
}

fn unpack_octave_data_8bit(msg: &[u8]) -> Result<(BoardIndex, Vec<u8>), LumatoneMidiError> {
  let msg = strip_sysex_markers(msg);
  let board_index = message_board_index(msg)?;
  let payload = message_payload(msg)?;
  Ok((board_index, unpack_8bit(payload)))
}

fn unpack_octave_data_7bit(msg: &[u8]) -> Result<(BoardIndex, Vec<u8>), LumatoneMidiError> {
  let msg = strip_sysex_markers(msg);
  let board_index = message_board_index(msg)?;
  let payload = message_payload(msg)?;
  Ok((board_index, payload.to_vec()))
}

fn unpack_channel_config_response(msg: &[u8]) -> Result<Response, LumatoneMidiError> {
  let board_index = message_board_index(msg)?;
  let payload = message_payload(msg)?;
  let mut channels = Vec::with_capacity(payload.len());
  for byte in payload {
    let ch = MidiChannel::try_from(*byte)?;
    channels.push(ch);
  }
  let response = Response::ChannelConfig(board_index, channels);
  Ok(response)
}

fn unpack_key_validity_response(msg: &[u8]) -> Result<Response, LumatoneMidiError> {
  let board_index = message_board_index(msg)?;
  let payload = message_payload(msg)?;
  let bools = payload.iter().map(|n| *n != 0).collect();
  Ok(Response::KeyValidity(board_index, bools))
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
    .map(|c| ((c[0] << 6) as u16) | (c[1] as u16))
    .collect()
}

/// Generic unpacking of 12-bit data from a SysEx message, when packed with three 4-bit values
fn unpack_12bit_from_4bit(payload: &[u8]) -> Vec<u16> {
  payload
    .chunks_exact(3)
    .map(|c| ((c[0] << 8) as u16) | ((c[1] << 4) as u16) | (c[2] as u16))
    .collect()
}

// endregion
