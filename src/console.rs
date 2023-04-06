use std::borrow::Cow;
use std::future::Future;
use std::sync::mpsc::{channel, SendError, Sender};
use std::time::Duration;

use console::style;
use indicatif::{ProgressBar, ProgressStyle};
use tokio::task::JoinError;

fn create_spinner(message: impl Into<Cow<'static, str>>) -> ProgressBar {
    let spinner = ProgressStyle::with_template("{prefix} {spinner} {wide_msg}")
        .unwrap()
        .tick_chars("⠁⠂⠄⡀⢀⠠⠐⠈ ");

    ProgressBar::new(u64::MAX)
        .with_style(spinner)
        .with_prefix(format!(" {}", style("WAIT").dim()))
        .with_message(message)
}

#[derive(Debug)]
pub enum Message {
    UpdateText(String),
    Finish,
}

pub struct Spinner {
    inner: ProgressBar,
}

impl Spinner {
    pub fn new(message: impl Into<Cow<'static, str>>) -> Self {
        Self {
            inner: create_spinner(message),
        }
    }

    pub async fn spin_while<F, Fut, T, E>(self, f: F) -> Result<T, E>
    where
        F: FnOnce(Sender<Message>) -> Fut,
        Fut: Future<Output = Result<T, E>>,
        E: From<SendError<Message>> + From<JoinError>,
    {
        let (tx, rx) = channel();
        let progress = tokio::spawn(async move {
            let p = self.inner;
            match rx.recv_timeout(Duration::from_millis(100)) {
                Ok(Message::UpdateText(message)) => p.set_message(message),
                Ok(Message::Finish) => {
                    p.finish_and_clear();

                    return;
                }
                Err(_) => (),
            }

            p.tick();
        });

        let res = f(tx.clone()).await;

        tx.send(Message::Finish)?;
        progress.await?;

        res
    }
}
