use uuid::Uuid;
use crate::capabilities::detect::LumatoneDeviceDescriptor;
use crate::capabilities::io::IncomingSysex;
use crate::capabilities::timeout::TimeoutId;
use crate::commands::Command;

pub enum Event {
  /// The shell has discovered a Lumatone device
  DeviceConnected {
    device: LumatoneDeviceDescriptor,
  },

  /// A connected device has disconnected
  DeviceDisconnected,

  /// The shell (or another part of the core) wants to send a Command message to the device
  CommandSubmitted(Command),

  /// The shell has received a Lumatone Sysex message on the Midi input channel
  SysexReceived(IncomingSysex),

  /// A timeout has been created
  TimeoutCreated(TimeoutId),
  /// A timeout has been cancelled
  TimeoutCanceled(TimeoutId),
  /// A timeout has triggered
  TimeoutElapsed(TimeoutId),

  // ----------------------------------------------------
  // internal events, dispatched from the core to itself
  // ----------------------------------------------------

  /// The device has indicated that it's busy, and we should try again later
  #[serde(skip)]
  DeviceBusy,

  /// The device failed to respond within the response timeout
  #[serde(skip)]
  ResponseTimedOut,

  /// The retry timeout has elapsed, and we're ready to retry sending the last command
  #[serde(skip)]
  ReadyToRetry,

  /// We've processed the response to a command and are ready to
  /// advance to the next command in the queue (if any)
  #[serde(skip)]
  ResponseProcessed,

  /// The send queue has been emptied, and we can transition back to the idle state
  #[serde(skip)]
  QueueEmpty,
}



pub struct Model {
  // TODO: add connection & driver state
}

#[derive(Default)]
pub struct MidiApp;
