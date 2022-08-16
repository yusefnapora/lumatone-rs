use super::sysex::EncodedSysex;

use log::warn;

enum State {
  Idle,
  ProcessingQueue { send_queue: Vec<EncodedSysex> },
  AwaitingResponse { send_queue: Vec<EncodedSysex>, command_sent: EncodedSysex },
  DeviceBusy { send_queue: Vec<EncodedSysex>, to_retry: EncodedSysex }
}

enum Action {
  SubmitCommand(EncodedSysex),
  MessageSent { msg: EncodedSysex, send_queue: Vec<EncodedSysex> },
  MessageReceived(EncodedSysex),
  ResponseTimedOut,
  ReadyToRetry,
}


impl State {

  fn next(self, action: Action) -> State {
    use State::*;
    use Action::*;

    match (action, self) {
      (SubmitCommand(msg), Idle) => {
        // Queue up message to send, switch to "processing state"
        ProcessingQueue { send_queue: vec![msg] }
      },

      (SubmitCommand(msg), AwaitingResponse { send_queue , command_sent }) => {
        // add new command to the send_queue
        let mut q = send_queue.clone();
        q.push(msg);
        AwaitingResponse { send_queue: q, command_sent: command_sent }
      },

      (SubmitCommand(msg), DeviceBusy { send_queue, to_retry }) => {
        // add new command to the send queue
        let mut q = send_queue.clone();
        q.push(msg);
        DeviceBusy { send_queue: q, to_retry: to_retry }
      },

      (MessageSent { msg, send_queue}, ProcessingQueue { send_queue: _old_queue }) => {
        AwaitingResponse { send_queue: send_queue, command_sent: msg }
      },

      (MessageReceived(msg), AwaitingResponse { send_queue, command_sent }) => {
        // TODO: check if received message is in response to command_sent
        //       if so, notify / log success
        //       if not, notify / log unexpected message
        //       if response says device is busy, enter DeviceBusy state

        if send_queue.is_empty() {
          Idle
        } else {
          ProcessingQueue { send_queue: send_queue }
        }
      },

      (MessageReceived(msg), state) => {
        warn!("Message received when not awaiting response: {:?}", msg);
        state
      },

      (ResponseTimedOut, AwaitingResponse { send_queue, command_sent }) => {
        warn!("Timed out waiting for response to msg: {:?}", command_sent);

        if send_queue.is_empty() {
          Idle
        } else {
          ProcessingQueue { send_queue: send_queue }
        }
      },

      (ResponseTimedOut, state) => {
        warn!("Response timeout action received, but not awaiting response");
        state
      },

      (ReadyToRetry, DeviceBusy { send_queue, to_retry }) => {
        let mut q = vec![to_retry];
        q.extend(send_queue);

        ProcessingQueue { send_queue: q }
      },

      (ReadyToRetry, state) => {
        warn!("ReadyToRetry action received but not in DeviceBusy state");
        state
      }

      _ => {
        panic!("invalid state transition"); // TODO better error handling
      }
    }
  }

  fn run(&mut self) { 
    use Action::*;
    use State::*;

    let action: Option<Action> = match &*self {
      Idle => { None },
      ProcessingQueue { send_queue } => {
        let msg = send_queue[0].clone();
        // TODO: send message on Midi output

        let remaining = &send_queue[1..];
        Some(MessageSent { msg, send_queue: remaining.to_vec() })
        }
      
      _ => { None }      
    };


  }
}
