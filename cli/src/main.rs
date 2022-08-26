mod cmd;

use crate::cmd::CliCommand;

use clap::Parser;
use tokio;

#[derive(Parser)]
#[clap(version, about, long_about = None)]
struct Cli {
  #[clap(subcommand)]
  command: CliCommand,
}

#[tokio::main]
async fn main() {
  let default_log_level = "debug";
  let env = env_logger::Env::default().filter_or("RUST_LOG", default_log_level);
  env_logger::init_from_env(env);

  let cli = Cli::parse();
  cli.command.run().await;
}
