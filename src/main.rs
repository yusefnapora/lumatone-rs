mod midi;

use tokio;

#[tokio::main]
async fn main() {
    println!("hi there! starting device detection");
    match midi::detect::detect_device().await {
      Err(e) => println!("midi detection error: {}", e),
      Ok(dev) => println!("found device: {:?}", dev)
    }
}
