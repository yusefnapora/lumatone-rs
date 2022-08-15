
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
