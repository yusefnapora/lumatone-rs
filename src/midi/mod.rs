mod commands;
mod constants;
mod sysex;
use midir::{MidiOutput, MidiInput};

pub struct LumatoneDevice {
  
}

pub fn detect_device() -> Result<LumatoneDevice, Box<dyn Error>> {
  // TODO: connect to all input devices, then connect to all output devices and
  // send an echo request packet, with some kind of id in the packet. If / when
  // we get an echo reply, use the packet to figure out which output device it
  // came from. Close all connections, but keep a reference to the in / out ports
  // in a LumatoneDevice struct and return it
  Err("not implemented".into())
}