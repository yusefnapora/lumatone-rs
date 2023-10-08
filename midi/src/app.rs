use crux_core::App;
use uuid::Uuid;
use serde::{Serialize, Deserialize};

use crate::capabilities::detect::LumatoneDeviceDescriptor;
use crate::capabilities::MidiCapabilities;
use crate::capabilities::timeout::TimeoutId;
use crate::commands::Command;
use crate::sysex::EncodedSysex;

type CommandSubmissionId = Uuid;

#[derive(Serialize, Deserialize, Clone, PartialEq, Eq)]
pub enum Event {
  /// The shell has discovered a Lumatone device
  DeviceConnected {
    device: LumatoneDeviceDescriptor,
  },

  /// A connected device has disconnected
  DeviceDisconnected,

  /// The shell (or another part of the core) wants to send a Command message to the device
  CommandSubmission {
    command: Command,
    id: CommandSubmissionId,
  },

  /// The shell has received a Lumatone Sysex message on the Midi input channel
  SysexReceived(EncodedSysex),

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


#[derive(Default)]
pub struct Model {
  // TODO: add connection & driver state
}

#[derive(Serialize, Deserialize)]
pub struct ViewModel {
  // TODO: add connection info, etc
}

#[derive(Default)]
pub struct MidiApp;

impl App for MidiApp {
  type Event = Event;
  type Model = Model;
  type ViewModel = ViewModel;
  type Capabilities = MidiCapabilities<Event>;

  fn update(&self, event: Self::Event, model: &mut Self::Model, caps: &Self::Capabilities) {
    todo!()
  }

  fn view(&self, model: &Self::Model) -> Self::ViewModel {
    ViewModel{}
  }
}