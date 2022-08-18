mod midi;

use std::time::Duration;

use crate::midi::commands::{set_key_color, set_key_function};
use crate::midi::constants::{
  key_loc_unchecked, LumatoneKeyFunction::NoteOnOff, MidiChannel, RGBColor,
};
use crate::midi::detect::detect_device;
use crate::midi::driver::MidiDriver;

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

  let commands = vec![
    set_key_color(key_loc_unchecked(1, 0), RGBColor::red()),
    set_key_color(key_loc_unchecked(1, 1), RGBColor::green()),
    set_key_color(key_loc_unchecked(1, 2), RGBColor::blue()),
    set_key_function(
      key_loc_unchecked(1, 0),
      NoteOnOff {
        channel,
        note_num: 50,
      },
    ),
    set_key_function(
      key_loc_unchecked(1, 1),
      NoteOnOff {
        channel,
        note_num: 51,
      },
    ),
    set_key_function(
      key_loc_unchecked(1, 2),
      NoteOnOff {
        channel,
        note_num: 52,
      },
    ),
  ];

  debug!("sending commands");
  for c in commands {
    debug!("sending command");
    command_tx.send(c).await.expect("send error");
  }

  tokio::time::sleep(Duration::from_secs(30)).await;

  debug!("sending done signal");
  done_tx.send(()).expect("error sending done signal");
  tokio::join!(h).0.expect("error joining driver future");
}
