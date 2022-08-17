mod midi;

use std::time::Duration;

use midi::constants::BoardIndex;
use midi::commands::set_key_light_parameters;
use midi::{detect::detect_device, error::LumatoneMidiError};
use midi::driver::MidiDriver;

use tokio;
use env_logger;
use log::{error, info, debug};

#[tokio::main]
async fn main() {
  let default_log_level = "debug";
  let env = env_logger::Env::default()
    .filter_or("RUST_LOG", default_log_level);
  
  env_logger::init_from_env(env);

  // TODO: fix lifetime issues... maybe tokio scopes?
  let device = detect_device().await.expect("device detection failed");
  let mut driver = MidiDriver::new(&device).expect("driver creation failed");

  let (done_tx, mut done_rx) = tokio::sync::oneshot::channel();
  let (command_tx, mut command_rx) = tokio::sync::mpsc::channel(128);
  let f = driver.run(command_rx, done_rx);

  debug!("starting driver loop");
  let h = tokio::spawn(f);
  debug!("driver loop spawned");

  let commands = vec![
    set_key_light_parameters(BoardIndex::Octave1, 0, 0xff, 0, 0),
    set_key_light_parameters(BoardIndex::Octave1, 1, 0, 0xff, 0),
    set_key_light_parameters(BoardIndex::Octave1, 2, 0, 0, 0xff),
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