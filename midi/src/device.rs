#![allow(dead_code)]
use serde::{Serialize, Deserialize}; 

/// Identifies the MIDI input and output ports that the Lumatone is connected to.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LumatoneDevice {
  out_port_name: String,
  in_port_name: String,
}

impl LumatoneDevice {
	pub fn new<S: AsRef<str>>(out_port_name: S, in_port_name: S) -> Self { 
		LumatoneDevice {
			out_port_name: out_port_name.as_ref().to_string(),
			in_port_name: in_port_name.as_ref().to_string(),
		}
	}
}
