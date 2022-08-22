mod keymap;
mod midi;

use std::time::Duration;

use crate::midi::commands::set_key_color;
use crate::midi::constants::{LumatoneKeyLocation, RGBColor};
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
  let (driver, driver_future) = MidiDriver::new(&device).expect("driver creation failed");

  debug!("starting driver loop");
  let h = tokio::spawn(driver_future);
  debug!("driver loop spawned");

  let commands = LumatoneKeyLocation::all().into_iter()
    .map(|loc| set_key_color(loc, RGBColor::green()));

  debug!("sending commands");
  for c in commands {
    debug!("sending command");
    driver.send(c).await.expect("send error");
  }

  tokio::time::sleep(Duration::from_secs(30)).await;

  debug!("sending done signal");
  driver.done().await.expect("error sending done signal");
  tokio::join!(h).0.expect("error joining driver future");
}
