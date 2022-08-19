/// Utilities for working with the .ltn Lumatone preset file format.
/// 

use std::{collections::HashMap, hash::Hash};
use crate::midi::constants::{LumatoneKeyLocation, LumatoneKeyFunction, RGBColor, BoardIndex};

use ini::Ini;
use num_traits::FromPrimitive;

pub struct KeyDefinition {
  function: LumatoneKeyFunction,
  color: RGBColor,
}

pub struct GeneralOptions {
  after_touch_active: bool,
  light_on_key_strokes: bool,
  invert_foot_controller: bool,
  invert_sustain: bool,
  expression_controller_sensitivity: u8,

  // TODO: velocity curves
  // TODO: fader config
  // TODO: aftertouch config
  // TODO: lumatouch config
}

impl Default for GeneralOptions {
  fn default() -> Self {
    GeneralOptions { 
      after_touch_active: false, 
      light_on_key_strokes: true, 
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