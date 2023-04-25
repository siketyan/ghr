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

use std::process::exit;

use clap::Parser;
use tracing::error;

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
    if let Err(e) = Cli::parse().run().await {
        error!("{}", e);
        exit(1);
    }
}
