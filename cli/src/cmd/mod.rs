mod debug;
mod send_preset;

use clap::Subcommand;
use std::path::PathBuf;

use self::{debug::run_debug_cmd, send_preset::run_send_preset};

#[derive(Subcommand)]
pub enum CliCommand {
  /// Does quick sanity-check debugging stuff. Actual behavior subject to change as I muck with things.
  Debug,

  /// Sends a .ltn preset file to the device
  SendPreset {
    #[clap(value_parser)]
    preset: PathBuf
  }
}

impl CliCommand {
  pub async fn run(&self) {
    match self {
      Self::Debug => run_debug_cmd().await,

      Self::SendPreset { preset } => run_send_preset(preset).await,
    }
  }
}