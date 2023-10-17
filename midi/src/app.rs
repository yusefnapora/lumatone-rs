use std::time::Duration;
use crux_core::App;
use uuid::Uuid;
use serde::{Serialize, Deserialize};
use log::debug;

use crate::capabilities::detect::LumatoneDeviceDescriptor;
use crate::capabilities::MidiCapabilities;
use crate::capabilities::timeout::TimeoutId;
use crate::commands::Command;
use crate::driver::actions::Action;
use crate::driver::effects::Effect;
use crate::driver::state::State;
use crate::driver::submission::CommandSubmission;
use crate::error::LumatoneMidiError;
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

  SysexSent(Result<(), LumatoneMidiError>),

  /// The shell has received a Lumatone Sysex message on the Midi input channel
  SysexReceived(EncodedSysex),

  /// A timeout has triggered
  TimeoutElapsed(TimeoutId),
}


#[derive(Default)]
pub struct Model {
  device: Option<LumatoneDeviceDescriptor>,
  driver_state: State,
  receive_timeout_id: Option<TimeoutId>,
  retry_timeout_id: Option<TimeoutId>,
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
    match event {
      Event::DeviceConnected { device } => {
        model.device = Some(device);
      }

      Event::DeviceDisconnected => {
        model.device = None;
        model.driver_state = State::Idle;
        model.receive_timeout_id = None;
        model.retry_timeout_id = None;
      }

      Event::CommandSubmission { command, id } => {
        let action = Action::SubmitCommand(CommandSubmission { command, submission_id: id });
        self.handle_driver_action(action, model, caps);
      }

      Event::SysexReceived(msg) => {
        let action = Action::MessageReceived(msg);
        self.handle_driver_action(action, model, caps);
      }

      Event::TimeoutElapsed(id) => {
        if model.retry_timeout_id == Some(id) {
          model.retry_timeout_id = None;
          self.handle_driver_action(Action::ReadyToRetry, model, caps);
        } else if model.receive_timeout_id == Some(id) {
          model.receive_timeout_id = None;
          self.handle_driver_action(Action::ResponseTimedOut, model, caps);
        } else {
          debug!("unknown timeout elapsed. timeout id: {}", id);
        }
      }
      
      Event::SysexSent(result) => {
        debug!("sysex send result: {:?}", result);
      }
    }
  }

  fn view(&self, _model: &Self::Model) -> Self::ViewModel {
    ViewModel{}
  }


}

impl MidiApp {
  fn handle_driver_action(&self, action: Action, model: &mut <MidiApp as App>::Model, caps: &<MidiApp as App>::Capabilities) {
    let current = model.driver_state.clone();
    model.driver_state = current.next(action);
    if let Some(effect) = model.driver_state.enter() {
      self.handle_driver_effect(effect, model, caps);
    }
  }

  fn handle_driver_effect(&self, effect: Effect, model: &mut <MidiApp as App>::Model, caps: &<MidiApp as App>::Capabilities) {
    match effect {
      Effect::SendMidiMessage(msg) => {
        caps.sysex.send(msg.command.to_sysex_message(), Event::SysexSent);
      }

      Effect::StartReceiveTimeout => {
        let duration = Duration::from_secs(1); // TODO: make configurable
        let id = caps.timeout.set(duration, Event::TimeoutElapsed);
        model.receive_timeout_id = Some(id);
      }

      Effect::StartRetryTimeout => {
        let duration = Duration::from_secs(1); // TODO: make configurable
        let id = caps.timeout.set(duration, Event::TimeoutElapsed);
        model.retry_timeout_id = Some(id);
      }

      Effect::NotifyMessageResponse(submission, result) => {

        caps.notify.send_command_result(submission.submission_id, result);
      }

      Effect::DispatchAction(action) => {
        self.handle_driver_action(action, model, caps);
      }
    }
  }
}