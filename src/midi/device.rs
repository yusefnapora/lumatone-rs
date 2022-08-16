#![allow(dead_code)]

#[derive(Debug)]
pub struct LumatoneDevice {
  out_port_name: String,
  in_port_name: String, 
}

impl LumatoneDevice {
  pub fn new(output_port_name: &str, input_port_name: &str) -> LumatoneDevice {
    LumatoneDevice {
      out_port_name: output_port_name.to_string(),
      in_port_name: input_port_name.to_string(),
    }
  }
}
