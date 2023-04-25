use std::borrow::Cow;
use std::future::Future;
use std::sync::mpsc::{channel, SendError};
use std::time::Duration;

use console::style;
use indicatif::{MultiProgress, ProgressBar, ProgressStyle};
use tokio::task::{JoinError, JoinHandle};

fn create_spinner(message: impl Into<Cow<'static, str>>) -> ProgressBar {
    let spinner = ProgressStyle::with_template("{prefix} {spinner} {wide_msg}")
        .unwrap()
        .tick_chars("⠁⠂⠄⡀⢀⠠⠐⠈ ");

    ProgressBar::new(u64::MAX)
        .with_style(spinner)
        .with_prefix(format!(" {}", style("WAIT").dim()))
        .with_message(message)
}

async fn spin_while<F, Fut, T, E>(p: ProgressBar, f: F) -> Result<T, E>
where
    F: FnOnce() -> Fut,
    Fut: Future<Output = Result<T, E>>,
    E: From<SendError<()>> + From<JoinError>,
{
    let (tx, rx) = channel();
    let progress = std::thread::spawn(move || {
        while rx.recv_timeout(Duration::from_millis(100)).is_err() {
            p.tick();
        }

        p.finish_and_clear();
    });

    let res = f().await;

    tx.send(())?;
    progress.join().ok();

    res
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
        F: FnOnce() -> Fut,
        Fut: Future<Output = Result<T, E>>,
        E: From<SendError<()>> + From<JoinError>,
    {
        spin_while(self.inner, f).await
    }
}

pub struct MultiSpinner<T, E> {
    inner: MultiProgress,
    handles: Vec<JoinHandle<Result<T, E>>>,
}

impl<T, E> MultiSpinner<T, E>
where
    T: Send + 'static,
    E: Send + 'static,
{
    pub fn new() -> Self {
        Self {
            inner: MultiProgress::new(),
            handles: Vec::new(),
        }
    }

    pub fn with_spin_while<F, Fut>(mut self, message: impl Into<Cow<'static, str>>, f: F) -> Self
    where
        F: FnOnce() -> Fut + Send + 'static,
        Fut: Future<Output = Result<T, E>> + Send + 'static,
        E: From<SendError<()>> + From<JoinError>,
    {
        let p = self.inner.add(create_spinner(message));
        self.handles.push(tokio::spawn(spin_while(p, f)));
        self
    }

    pub async fn collect(self) -> Result<Vec<T>, E>
    where
        E: From<JoinError> + From<std::io::Error>,
    {
        let mut results = Vec::with_capacity(self.handles.len());
        for h in self.handles {
            results.push(h.await??);
        }

        self.inner.clear()?;

        Ok(results)
    }
}
