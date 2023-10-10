//! Crux capability definitions for Lumatone MIDI operations

pub mod detect;
pub mod connect;
pub mod io;
pub mod timeout;
mod response;

pub struct MidiCapabilities<Ev> {
  detect: detect::DetectDevice<Ev>,
  connect: connect::ConnectToDevice<Ev>,
  sysex: io::Sysex<Ev>,
}