mod application;
mod cmd;
mod config;
mod console;
mod git;
mod path;
mod platform;
mod profile;
mod repository;
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

const BUILD_INFO: &str = build_info::format!(
    "{} v{} built with {} at {}",
    $.crate_info.name,
    $.crate_info.version,
    $.compiler,
    $.timestamp,
);

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
