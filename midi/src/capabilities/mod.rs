//! Crux capability definitions for Lumatone MIDI operations

pub mod detect;
mod connect;
mod io;

pub struct MidiCapabilities<Ev> {
  detect: detect::DetectDevice<Ev>,
  connect: connect::ConnectToDevice<Ev>,
  send_sysex: io::SendSysex<Ev>,
  receive_sysex_stream: io::ReceiveSysexStream<Ev>,
}