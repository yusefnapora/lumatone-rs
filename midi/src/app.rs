use crate::commands::Command;
use crate::sysex::EncodedSysex;

pub enum Event {
    DeviceConnected(LumatoneConnection),
    DeviceDisconnected(LumatoneConnection),


    CommandSubmitted(LumatoneConnection, Command),
    SysexReceived(LumatoneConnection, EncodedSysex),
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
