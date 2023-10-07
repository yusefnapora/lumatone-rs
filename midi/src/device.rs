#![allow(dead_code)]


/// Identifies the MIDI input and output ports that the Lumatone is connected to.
/// A LumatoneDevice can be used to initiate a connection to the device using [`Self::connect`].
#[derive(Debug, Clone)]
pub struct LumatoneDevice {
  out_port_name: String,
  in_port_name: String,
}
