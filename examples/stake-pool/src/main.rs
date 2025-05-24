use std::path::PathBuf;

use clap::Parser;
use stake_pool_parser::{accounts_parser::AccountParser, instructions_parser::InstructionParser};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};
use yellowstone_vixen::{self as vixen, Pipeline};

#[derive(clap::Parser)]
#[command(version, author, about)]
pub struct Opts {
    #[arg(long, short)]
    config: PathBuf,
}

#[derive(Debug)]
pub struct Logger;

impl<V: std::fmt::Debug + Sync> vixen::Handler<V> for Logger {
    async fn handle(&self, value: &V) -> vixen::HandlerResult<()> {
        tracing::info!(?value);
        Ok(())
    }
}

fn main() {
    tracing_subscriber::registry()
        .with(tracing_subscriber::EnvFilter::from_default_env())
        .with(tracing_subscriber::fmt::layer())
        .init();

    let Opts { config } = Opts::parse();
    let config = std::fs::read_to_string(config).expect("Error reading config file");
    let config = toml::from_str(&config).expect("Error parsing config");

    yellowstone_vixen::Runtime::builder()
        .account(Pipeline::new(AccountParser, [Logger]))
        .instruction(Pipeline::new(InstructionParser, [Logger]))
        .commitment_level(yellowstone_vixen::CommitmentLevel::Confirmed)
        .build(config)
        .run();
}
