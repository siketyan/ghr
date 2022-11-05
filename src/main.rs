mod cmd;
mod config;
mod path;
mod profile;
mod root;
mod rule;
mod url;

use clap::Parser;
use std::process::exit;
use tracing::error;
use tracing_subscriber::filter::LevelFilter;
use tracing_subscriber::EnvFilter;

use crate::cmd::Cli;

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt()
        .compact()
        .without_time()
        .with_target(false)
        .with_env_filter(
            EnvFilter::builder()
                .with_default_directive(LevelFilter::INFO.into())
                .from_env_lossy(),
        )
        .init();

    if let Err(e) = Cli::parse().run().await {
        error!("{}", e);
        exit(1);
    }
}
