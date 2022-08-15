#![allow(dead_code)]
use midir::{MidiOutput, MidiInput};


pub struct LumatoneDevice {
  output: MidiOutput,
  input: MidiInput,

  out_port_name: String,
  in_port_name: String, 
}

impl LumatoneDevice {
  pub fn new(midi_output: MidiOutput, midi_input: MidiInput, output_port_name: &str, input_port_name: &str) -> LumatoneDevice {
    LumatoneDevice {
      output: midi_output,
      input: midi_input,
      out_port_name: output_port_name.to_string(),
      in_port_name: input_port_name.to_string(),
    }
  }
}

impl std::fmt::Debug for LumatoneDevice {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("LumatoneDevice")
          .field("out_port_name", &self.out_port_name)
          .field("in_port_name", &self.in_port_name).finish()
    }
}