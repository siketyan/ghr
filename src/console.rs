use std::borrow::Cow;

use console::style;
use indicatif::{ProgressBar, ProgressStyle};

pub fn create_spinner(message: impl Into<Cow<'static, str>>) -> ProgressBar {
    let spinner = ProgressStyle::with_template("{prefix} {spinner} {wide_msg}")
        .unwrap()
        .tick_chars("⠁⠂⠄⡀⢀⠠⠐⠈ ");

    ProgressBar::new(u64::MAX)
        .with_style(spinner)
        .with_prefix(format!(" {}", style("WAIT").dim()))
        .with_message(message)
}
