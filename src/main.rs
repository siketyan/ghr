mod application;
mod cmd;
mod config;
mod console;
mod path;
mod profile;
mod root;
mod rule;
mod url;

use std::io::stderr;
use std::process::exit;

use clap::Parser;
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
        .with_writer(stderr)
        .init();

    if let Err(e) = Cli::parse().run().await {
        error!("{}", e);
        exit(1);
    }
}
