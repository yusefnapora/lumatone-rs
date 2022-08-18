mod midi;

use std::time::Duration;

use crate::midi::commands::{SetKeyColor, SetKeyFunction};
use crate::midi::driver::MidiDriver;
use crate::midi::detect::detect_device;
use crate::midi::constants::{LumatoneKeyFunction, MidiChannel, RGBColor, key_uncheked};

use env_logger;
use log::debug;
use tokio;

#[tokio::main]
async fn main() {
  let default_log_level = "debug";
  let env = env_logger::Env::default().filter_or("RUST_LOG", default_log_level);

  env_logger::init_from_env(env);

  let device = detect_device().await.expect("device detection failed");
  let driver = MidiDriver::new(&device).expect("driver creation failed");

  let (done_tx, done_rx) = tokio::sync::oneshot::channel();
  let (command_tx, command_rx) = tokio::sync::mpsc::channel(128);
  let f = driver.run(command_rx, done_rx);

  debug!("starting driver loop");
  let h = tokio::spawn(f);
  debug!("driver loop spawned");

  let channel = MidiChannel::default();
  // using two vectors for the commands to avoid a bunch of boxing...
  // maybe LumatoneCommand would be easier to use as an enum with struct variants
  // instead of a trait...
  let color_commands = vec![
    SetKeyColor::new(key_uncheked(1, 0), RGBColor::red()),
    SetKeyColor::new(key_uncheked(1, 1), RGBColor::green()),
    SetKeyColor::new(key_uncheked(1, 2), RGBColor::blue()),
  ];

  let function_commands = vec![
    SetKeyFunction::new(key_uncheked(1, 0), LumatoneKeyFunction::NoteOnOff { channel, note_num: 50 }),
    SetKeyFunction::new(key_uncheked(1, 1), LumatoneKeyFunction::NoteOnOff { channel, note_num: 51 }),
    SetKeyFunction::new(key_uncheked(1, 2), LumatoneKeyFunction::NoteOnOff { channel, note_num: 52 }),
  ];

  debug!("sending commands");
  for c in color_commands {
    debug!("sending command");
    command_tx.send(Box::new(c)).await.expect("send error");
  }

  for c in function_commands {
    command_tx.send(Box::new(c)).await.expect("send error");
  }

  tokio::time::sleep(Duration::from_secs(30)).await;

  debug!("sending done signal");
  done_tx.send(()).expect("error sending done signal");
  tokio::join!(h).0.expect("error joining driver future");
}
