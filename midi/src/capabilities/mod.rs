//! Crux capability definitions for Lumatone MIDI operations

pub mod detect;
pub mod connect;
pub mod io;
pub mod timeout;
pub mod notify;

pub struct MidiCapabilities<Ev> {
  pub detect: detect::DetectDevice<Ev>,
  pub connect: connect::ConnectToDevice<Ev>,
  pub sysex: io::Sysex<Ev>,
	pub notify: notify::NotifyShell<Ev>,
  pub timeout: timeout::Timeout<Ev>,
}
