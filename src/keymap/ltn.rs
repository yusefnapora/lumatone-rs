#![allow(unused)]
/// Utilities for working with the .ltn Lumatone preset file format.
/// 

use std::collections::HashMap;
use crate::midi::constants::{LumatoneKeyLocation, LumatoneKeyFunction, RGBColor, BoardIndex};

use ini::Ini;
use num_traits::FromPrimitive;

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

  // TODO: velocity curves
  // TODO: fader config
  // TODO: aftertouch config
  // TODO: lumatouch config
}

impl Default for GeneralOptions {
  fn default() -> Self {
    GeneralOptions { 
      after_touch_active: false, 
      light_on_key_strokes: false, 
      invert_foot_controller: false, 
      invert_sustain: false, 
      expression_controller_sensitivity: 0,
    }
  }
}

pub struct LumatoneKeyMap {
  keys: HashMap<LumatoneKeyLocation, KeyDefinition>,
  general: GeneralOptions,
}


impl LumatoneKeyMap {
  pub fn new() -> Self {
    LumatoneKeyMap { keys: HashMap::new(), general: GeneralOptions::default() }
  }

  pub fn set_key<'a>(&'a mut self, location: LumatoneKeyLocation, def: KeyDefinition) -> &'a mut LumatoneKeyMap {
    self.keys.insert(location, def);
    self
  }

  // TODO: add batch key update fn that takes HashMap or seq of (location, definition) tuples

  pub fn set_global_options<'a>(&'a mut self, opts: GeneralOptions) -> &'a mut LumatoneKeyMap {
    self.general = opts;
    self
  }

  

  pub fn as_ini(&self) -> Ini {
    let mut conf = Ini::new();

    let bool_str = |b: bool| if b { 1 } else { 0 }.to_string();
    // set general options
    conf.with_general_section()
        .set("AfterTouchActive", bool_str(self.general.after_touch_active))
        .set("LightOnKeyStrokes", bool_str(self.general.light_on_key_strokes))
        .set("InvertFootController", bool_str(self.general.invert_foot_controller))
        .set("InvertSustain", bool_str(self.general.invert_sustain))
        .set("ExprCtrlSensivity", self.general.expression_controller_sensitivity.to_string());
 
    // Key definitions are split into sections, one for each board / octave
    for b in 1 ..= 5 {
      let board_index: BoardIndex = FromPrimitive::from_u8(b).unwrap();
      let keys = self.keys.iter()
        .filter(|(loc, _)| loc.board_index() == board_index);

      for (loc, def) in keys {
        let key_index: u8 = loc.key_index().into();
        let key_type = def.function.key_type_code();
        let sec = format!("Board{b}");
        
        conf.with_section(Some(sec.clone()))
          .set(format!("Key_{key_index}"), def.function.note_or_cc_num().to_string())
          .set(format!("Chan_{key_index}"), def.function.midi_channel_byte().to_string())
          .set(format!("Col_{key_index}"), def.color.to_hex_string());

        if key_type != 1 {
          conf.with_section(Some(sec))
            .set(format!("KTyp_{key_index}"), key_type.to_string());
        }
      }     
    }

    conf
  }
}

#[cfg(test)]
mod tests {
    use crate::midi::constants::{LumatoneKeyFunction, MidiChannel, key_loc_unchecked, RGBColor};

    use super::{LumatoneKeyMap, KeyDefinition, GeneralOptions};


  #[test]
  fn test_keymap_to_ini() {

    let mut keymap = LumatoneKeyMap::new();

    keymap
      .set_key(key_loc_unchecked(1, 0), KeyDefinition { 
        function: LumatoneKeyFunction::NoteOnOff { channel: MidiChannel::default(), note_num: 60 },
        color: RGBColor(0xff, 0, 0)
      })
      .set_key(key_loc_unchecked(2, 0), KeyDefinition {
        function: LumatoneKeyFunction::LumaTouch { channel: MidiChannel::unchecked(1), note_num: 70, fader_up_is_null: false },
        color: RGBColor::green()
      });

    let ini = keymap.as_ini();
    let board_1 = ini.section(Some("Board1".to_string())).unwrap();
    assert_eq!(board_1.get("Key_0"), Some("60"));
    assert_eq!(board_1.get("Chan_0"), Some("0"));
    assert_eq!(board_1.get("Col_0"), Some("ff0000"));
    assert_eq!(board_1.get("KTyp_0"), None); // KTyp is only set if keytype is not NoteOnOff

    let board_2 = ini.section(Some("Board2".to_string())).unwrap();
    assert_eq!(board_2.get("Key_0"), Some("70"));
    assert_eq!(board_2.get("Chan_0"), Some("1"));
    assert_eq!(board_2.get("Col_0"), Some("00ff00"));
    assert_eq!(board_2.get("KTyp_0"), Some("3"));

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
    });

    let ini = keymap.as_ini();
    let general = ini.general_section();
    assert_eq!(general.get("AfterTouchActive"), Some("1"));
    assert_eq!(general.get("LightOnKeyStrokes"), Some("1"));
    assert_eq!(general.get("InvertFootController"), Some("1"));
    assert_eq!(general.get("InvertSustain"), Some("1"));
    assert_eq!(general.get("ExprCtrlSensivity"), Some("100"));
  }

}