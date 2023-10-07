use crate::commands::Command;
use crate::sysex::EncodedSysex;

pub enum Event {
    /// The shell has discovered a Lumatone device
    DeviceConnected(LumatoneConnection),

    /// A connected device has disconnected
    DeviceDisconnected(LumatoneConnection),

    /// The shell (or another part of the core) wants to send a Command message to the device
    CommandSubmitted(LumatoneConnection, Command),

    /// The shell has received a Lumatone Sysex message on the Midi input channel
    SysexReceived(LumatoneConnection, EncodedSysex),



    // internal events, dispatched from the core to itself
    DeviceBusy(LumatoneConnection),

}

pub struct LumatoneConnection {
    id: String,
}

// TODO: refactor the real midi driver state to remove non-serializable stuff
// and use it instead of this placeholder.
// I'm thinking that we should also add "disconnected" and "searching" states to the
// driver, instead of only creating a driver once we have a connection. That way we
// always have a DriverState, regardless of whether we have a connection. The connection
// id can just be part of the state struct for all the "connected" states.
pub enum DriverState {

}

pub struct Model {

    driver: DriverState
}

#[derive(Default)]
pub struct MidiApp;
