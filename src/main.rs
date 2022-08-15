mod midi;

use tokio;
use env_logger;
use log::{error, info};

#[tokio::main]
async fn main() {
    let default_log_level = "info";
    let env = env_logger::Env::default()
      .filter_or("RUST_LOG", default_log_level);
    
    env_logger::init_from_env(env);

    info!("hi there! starting device detection");
    match midi::detect::detect_device().await {
      Err(e) => error!("midi detection error: {}", e),
      Ok(dev) => info!("found device: {:?}", dev)
    }
}
