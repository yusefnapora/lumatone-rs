use std::fs;
use std::path::PathBuf;

use lumatone_core::keymap::ltn::LumatoneKeyMap;
use lumatone_core::midi::detect::detect_device;
use lumatone_core::midi::driver::MidiDriver;

pub async fn run_send_preset(path: &PathBuf) {
  let contents = fs::read_to_string(path).expect("unable to read preset");
  let keymap = LumatoneKeyMap::from_ini_str(contents).expect("unable to load presest");

  let device = detect_device().await.expect("device detection failed");
  let (driver, driver_future) = MidiDriver::new(&device).expect("driver creation failed");

  log::debug!("starting driver loop");
  let h = tokio::spawn(driver_future);
  log::debug!("driver loop spawned");

  let commands = keymap.to_midi_commands();
  log::debug!("sending commands");
  for c in commands {
    log::debug!("sending command {}", c);
    let res = driver.send(c).await;
    log::debug!("received response: {res:?}");
  }

  log::debug!("sending done signal");
  driver.done().await.expect("error sending done signal");
  tokio::join!(h).0.expect("error joining driver future");
}
