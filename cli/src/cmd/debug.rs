use lumatone_core::midi::{
  commands::set_key_color,
  constants::{LumatoneKeyLocation, RGBColor},
  detect::detect_device,
  driver::MidiDriver,
};

use log::debug;
use tokio;

pub async fn run_debug_cmd() {
  let device = detect_device().await.expect("device detection failed");
  let (driver, driver_future) = MidiDriver::new(&device).expect("driver creation failed");

  debug!("starting driver loop");
  let h = tokio::spawn(driver_future);
  debug!("driver loop spawned");

  let commands = LumatoneKeyLocation::all()
    .into_iter()
    .map(|loc| set_key_color(loc, RGBColor::random()));

  debug!("sending commands");
  for c in commands {
    debug!("sending command");
    let res = driver.send(c).await;
    debug!("received response: {res:?}");
  }

  debug!("sending done signal");
  driver.done().await.expect("error sending done signal");
  tokio::join!(h).0.expect("error joining driver future");
}
