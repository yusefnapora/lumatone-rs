#![allow(unused)]
use lumatone_midi::{constants::{
  key_loc_unchecked, BoardIndex, LumatoneKeyFunction, LumatoneKeyIndex, LumatoneKeyLocation,
  MidiChannel, RGBColor,
}, commands::Command};
/// Utilities for working with the .ltn Lumatone preset file format.
///
use std::collections::HashMap;

use ini::{Ini, Properties};
use num_traits::FromPrimitive;

use super::{
  error::LumatoneKeymapError,
  table_defaults,
  tables::{
    parse_velocity_intervals, velocity_intervals_to_string, ConfigTableDefinition,
    ConfigurationTables,
  },
};

pub struct KeyDefinition {
  pub function: LumatoneKeyFunction,
  pub color: RGBColor,
}

pub struct GeneralOptions {
  pub after_touch_active: bool,
  pub light_on_key_strokes: bool,
  pub invert_foot_controller: bool,
  pub invert_sustain: bool,
  pub expression_controller_sensitivity: u8,

  pub config_tables: ConfigurationTables,
}

fn config_table_from_ini_section(
  section: &Properties,
  key: &str,
) -> Result<Option<ConfigTableDefinition>, LumatoneKeymapError> {
  match section.get(key) {
    Some(val) => ConfigTableDefinition::from_str(val).map(|val| Some(val)),
    None => Ok(None),
  }
}

impl GeneralOptions {
  fn from_ini_section(props: &Properties) -> Result<GeneralOptions, LumatoneKeymapError> {
    let on_off_velocity = config_table_from_ini_section(props, "NoteOnOffVelocityCurveTbl")?;
    let fader_velocity = config_table_from_ini_section(props, "FaderConfig")?;
    let aftertouch_velocity = config_table_from_ini_section(props, "afterTouchConfig")?;
    let lumatouch_velocity = config_table_from_ini_section(props, "LumaTouchConfig")?;
    let velocity_intervals = match props.get("VelocityIntrvlTbl") {
      Some(val) => Some(parse_velocity_intervals(val)?),
      None => None,
    };

    Ok(GeneralOptions {
      after_touch_active: props.get("AfterTouchActive").map(bool_val).unwrap_or(false),
      light_on_key_strokes: props
        .get("LightOnKeyStrokes")
        .map(bool_val)
        .unwrap_or(false),
      invert_foot_controller: props
        .get("InvertFootController")
        .map(bool_val)
        .unwrap_or(false),
      invert_sustain: props.get("InvertSustain").map(bool_val).unwrap_or(false),
      // TODO: don't panic on invalid int:
      expression_controller_sensitivity: props
        .get("ExprCtrlSensivity")
        .map(|s| u8::from_str_radix(s, 10).expect("invalid int value"))
        .unwrap_or(0),
      config_tables: ConfigurationTables {
        on_off_velocity,
        fader_velocity,
        aftertouch_velocity,
        lumatouch_velocity,
        velocity_intervals,
      },
    })
  }
}

impl Default for GeneralOptions {
  fn default() -> Self {
    GeneralOptions {
      after_touch_active: false,
      light_on_key_strokes: false,
      invert_foot_controller: false,
      invert_sustain: false,
      expression_controller_sensitivity: 0,
      config_tables: ConfigurationTables::default(),
    }
  }
}

pub struct LumatoneKeyMap {
  keys: HashMap<LumatoneKeyLocation, KeyDefinition>,
  general: GeneralOptions,
}

impl LumatoneKeyMap {
  pub fn new() -> Self {
    LumatoneKeyMap {
      keys: HashMap::new(),
      general: GeneralOptions::default(),
    }
  }

  pub fn set_key<'a>(
    &'a mut self,
    location: LumatoneKeyLocation,
    def: KeyDefinition,
  ) -> &'a mut LumatoneKeyMap {
    self.keys.insert(location, def);
    self
  }

  // TODO: add batch key update fn that takes HashMap or seq of (location, definition) tuples

  pub fn set_global_options<'a>(&'a mut self, opts: GeneralOptions) -> &'a mut LumatoneKeyMap {
    self.general = opts;
    self
  }

  pub fn to_ini(&self) -> Ini {
    let mut conf = Ini::new();

    let bool_str = |b: bool| if b { 1 } else { 0 }.to_string();
    // set general options
    conf
      .with_general_section()
      .set(
        "AfterTouchActive",
        bool_str(self.general.after_touch_active),
      )
      .set(
        "LightOnKeyStrokes",
        bool_str(self.general.light_on_key_strokes),
      )
      .set(
        "InvertFootController",
        bool_str(self.general.invert_foot_controller),
      )
      .set("InvertSustain", bool_str(self.general.invert_sustain))
      .set(
        "ExprCtrlSensivity",
        self.general.expression_controller_sensitivity.to_string(),
      );

    if let Some(t) = &self.general.config_tables.velocity_intervals {
      conf
        .with_general_section()
        .set("VelocityIntrvlTbl", velocity_intervals_to_string(t));
    }

    if let Some(t) = &self.general.config_tables.on_off_velocity {
      conf
        .with_general_section()
        .set("NoteOnOffVelocityCrvTbl", t.to_string());
    }

    if let Some(t) = &self.general.config_tables.fader_velocity {
      conf
        .with_general_section()
        .set("FaderConfig", t.to_string());
    }

    if let Some(t) = &self.general.config_tables.aftertouch_velocity {
      conf
        .with_general_section()
        .set("afterTouchConfig", t.to_string());
    }

    if let Some(t) = &self.general.config_tables.lumatouch_velocity {
      conf
        .with_general_section()
        .set("LumaTouchConfig", t.to_string());
    }

    // Key definitions are split into sections, one for each board / octave
    for b in 1..=5 {
      let board_index: BoardIndex = FromPrimitive::from_u8(b).unwrap();
      let keys = self
        .keys
        .iter()
        .filter(|(loc, _)| loc.board_index() == board_index);

      let section_name = format!("Board{}", b-1);
      for (loc, def) in keys {
        let key_index: u8 = loc.key_index().into();
        let key_type = def.function.key_type_code();

        conf
          .with_section(Some(section_name.clone()))
          .set(
            format!("Key_{key_index}"),
            def.function.note_or_cc_num().to_string(),
          )
          .set(
            format!("Chan_{key_index}"),
            def.function.midi_channel_byte().to_string(),
          )
          .set(format!("Col_{key_index}"), def.color.to_hex_string());

        if key_type != 1 {
          conf
            .with_section(Some(section_name.clone()))
            .set(format!("KTyp_{key_index}"), key_type.to_string());
        }
      }

      // explicitly set any missing keys to "disabled"
      for k in LumatoneKeyIndex::MIN_VALUE..=LumatoneKeyIndex::MAX_VALUE {
        let key_index = LumatoneKeyIndex::unchecked(k);
        let loc = LumatoneKeyLocation(board_index, key_index);
        if self.keys.contains_key(&loc) {
          continue;
        }
        conf
          .with_section(Some(section_name.clone()))
          .set(format!("Key_{key_index}"), "0")
          .set(format!("Chan_{key_index}"), "1")
          .set(format!("Col_{key_index}"), "000000")
          .set(format!("KTyp_{key_index}"), "4");
      }
    }

    conf
  }

  pub fn from_ini_str<S: AsRef<str>>(source: S) -> Result<LumatoneKeyMap, LumatoneKeymapError> {
    let ini = Ini::load_from_str(source.as_ref())?;

    let mut general = GeneralOptions::default();
    let mut keys: HashMap<LumatoneKeyLocation, KeyDefinition> = HashMap::new();

    for b in 1..=5 {
      let key = format!("Board{}", b-1);
      if let Some(section) = ini.section(Some(key)) {

        // The official LumatoneEditor just spits global options out at the end of the file,
        // so they get slurped into the [Board5] section.
        if let Ok(general_opts) = GeneralOptions::from_ini_section(section) {
          general = general_opts;
        }

        for k in 0..=55 {
          let key_type_code = get_u8_or_default_from_ini_section(section, format!("KTyp_{k}"), 1);
          let note_or_cc_num = get_u8_or_default_from_ini_section(section, format!("Key_{k}"), 0);
          let chan = get_u8_or_default_from_ini_section(section, format!("Chan_{k}"), 1);
          let color_str = section.get(format!("Col_{k}")).unwrap_or("000000");
          // TODO: use error_stack here:
          let color_u32 =
            u32::from_str_radix(color_str, 16).map_err(|_| LumatoneKeymapError::ValueParseError)?;
          let color = RGBColor::from(color_u32);

          let channel = MidiChannel::new(chan).unwrap_or_default();
          let function = match key_type_code {
            1 => LumatoneKeyFunction::NoteOnOff {
              channel,
              note_num: note_or_cc_num,
            },
            // FIXME: figure out how the fader up thing is serialized in preset files...
            // might be the same as in midi messages (left shift the type code by 4)
            2 => LumatoneKeyFunction::ContinuousController {
              channel,
              cc_num: note_or_cc_num,
              fader_up_is_null: false,
            },
            3 => LumatoneKeyFunction::LumaTouch {
              channel,
              note_num: note_or_cc_num,
              fader_up_is_null: false,
            },
            4 => LumatoneKeyFunction::Disabled,
            _ => {
              log::warn!("unrecognized key type code: {key_type_code}");
              LumatoneKeyFunction::Disabled
            }
          };
          let key_definition = KeyDefinition { function, color };
          let loc = key_loc_unchecked(b, k);
          keys.insert(loc, key_definition);
        }
      }
    }

    Ok(LumatoneKeyMap { keys, general })
  }

  pub fn to_midi_commands(&self) -> Vec<Command> {
    use Command::*;
    let mut commands = vec![
      SetAftertouchEnabled(self.general.after_touch_active),
      SetLightOnKeystrokes(self.general.light_on_key_strokes),
      InvertFootController(self.general.invert_foot_controller),
      InvertSustainPedal(self.general.invert_sustain),
      SetExpressionPedalSensitivity(self.general.expression_controller_sensitivity),
    ];

    let tables = &self.general.config_tables;
    if let Some(t) = &tables.on_off_velocity {
      commands.push(SetVelocityConfig(Box::new(t.table)));
    }
    if let Some(t) = &tables.aftertouch_velocity {
      commands.push(SetAftertouchConfig(Box::new(t.table)));
    }
    if let Some(t) = &tables.fader_velocity {
      commands.push(SetFaderConfig(Box::new(t.table)));
    }
    if let Some(t) = &tables.lumatouch_velocity {
      commands.push(SetLumatouchConfig(Box::new(t.table)));
    }
    if let Some(t) = tables.velocity_intervals {
      commands.push(SetVelocityIntervals(Box::new(t)));
    }

    for (location, definition) in self.keys.iter() {
      commands.push(SetKeyFunction { location: *location, function: definition.function });
      commands.push(SetKeyColor { location: *location, color: definition.color });
    }

    commands
  }
}


fn bool_val(s: &str) -> bool {
  let i = i64::from_str_radix(s, 10).unwrap_or(0);
  i != 0
}

fn get_u8_or_default_from_ini_section<S: AsRef<str>>(
  section: &Properties,
  key: S,
  default_val: u8,
) -> u8 {
  section
    .get(key)
    .map(|v| u8::from_str_radix(v, 10).unwrap_or(default_val))
    .unwrap_or(default_val)
}

#[cfg(test)]
mod tests {
  use crate::tables::ConfigurationTables;
  use lumatone_midi::constants::{key_loc_unchecked, LumatoneKeyFunction, MidiChannel, RGBColor};

  use super::{GeneralOptions, KeyDefinition, LumatoneKeyMap};

  #[test]
  fn test_keymap_to_ini() {
    let mut keymap = LumatoneKeyMap::new();

    keymap
      .set_key(
        key_loc_unchecked(1, 0),
        KeyDefinition {
          function: LumatoneKeyFunction::NoteOnOff {
            channel: MidiChannel::default(),
            note_num: 60,
          },
          color: RGBColor(0xff, 0, 0),
        },
      )
      .set_key(
        key_loc_unchecked(2, 0),
        KeyDefinition {
          function: LumatoneKeyFunction::LumaTouch {
            channel: MidiChannel::unchecked(2),
            note_num: 70,
            fader_up_is_null: false,
          },
          color: RGBColor::green(),
        },
      );

    let ini = keymap.to_ini();
    let board_1 = ini.section(Some("Board1".to_string())).unwrap();
    assert_eq!(board_1.get("Key_0"), Some("60"));
    assert_eq!(board_1.get("Chan_0"), Some("1"));
    assert_eq!(board_1.get("Col_0"), Some("ff0000"));
    assert_eq!(board_1.get("KTyp_0"), None); // KTyp is only set if keytype is not NoteOnOff

    let board_2 = ini.section(Some("Board2".to_string())).unwrap();
    assert_eq!(board_2.get("Key_0"), Some("70"));
    assert_eq!(board_2.get("Chan_0"), Some("2"));
    assert_eq!(board_2.get("Col_0"), Some("00ff00"));
    assert_eq!(board_2.get("KTyp_0"), Some("3"));

    // missing keys should have KTyp == 4 (disabled), Key = 0, Chan = 1, Col = 000000
    let board_3 = ini.section(Some("Board3".to_string())).unwrap();
    assert_eq!(board_3.get("Key_10"), Some("0"));
    assert_eq!(board_3.get("Chan_10"), Some("1"));
    assert_eq!(board_3.get("Col_10"), Some("000000"));
    assert_eq!(board_3.get("KTyp_10"), Some("4"));

    let general = ini.general_section();
    assert_eq!(general.get("AfterTouchActive"), Some("0"));
    assert_eq!(general.get("LightOnKeyStrokes"), Some("0"));
    assert_eq!(general.get("InvertFootController"), Some("0"));
    assert_eq!(general.get("InvertSustain"), Some("0"));
    assert_eq!(general.get("ExprCtrlSensivity"), Some("0"));
  }

  #[test]
  fn test_general_opts_to_ini() {
    let mut keymap = LumatoneKeyMap::new();

    keymap.set_global_options(GeneralOptions {
      after_touch_active: true,
      light_on_key_strokes: true,
      invert_foot_controller: true,
      invert_sustain: true,
      expression_controller_sensitivity: 100,
      config_tables: ConfigurationTables::default(),
    });

    let ini = keymap.to_ini();
    let general = ini.general_section();
    assert_eq!(general.get("AfterTouchActive"), Some("1"));
    assert_eq!(general.get("LightOnKeyStrokes"), Some("1"));
    assert_eq!(general.get("InvertFootController"), Some("1"));
    assert_eq!(general.get("InvertSustain"), Some("1"));
    assert_eq!(general.get("ExprCtrlSensivity"), Some("100"));
  }
}
