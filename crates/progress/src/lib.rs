//! Terminal progress bars and spinners for long-running operations.
//!
//! Wraps `indicatif` to provide a consistent progress-reporting API
//! for downloads, installations, and other batch operations.

use std::time::{Duration, Instant};

/// Error type for progress operations.
#[derive(Debug, thiserror::Error)]
pub enum ProgressError {
    #[error("progress bar error")]
    Internal,
}

/// A single progress bar tracking completion of a unit of work.
pub struct ProgressBar {
    inner: Option<indicatif::ProgressBar>,
    total: u64,
    completed: u64,
    message: String,
}

impl ProgressBar {
    /// Create a new progress bar with the given total.
    pub fn new(total: u64) -> Self {
        let bar = indicatif::ProgressBar::new(total);
        bar.set_style(
            indicatif::ProgressStyle::default_bar()
                .template("{spinner:.green} [{elapsed_precise}] [{bar:40.cyan/blue}] {pos}/{len} ({eta}) {msg}")
                .unwrap()
                .progress_chars("#>-"),
        );
        Self {
            inner: Some(bar),
            total,
            completed: 0,
            message: String::new(),
        }
    }

    /// Create a new hidden (non-visible) progress bar.
    pub fn new_hidden(total: u64) -> Self {
        let bar = indicatif::ProgressBar::new(total);
        bar.set_style(indicatif::ProgressStyle::default_bar().template("{msg}").unwrap());
        Self {
            inner: Some(bar),
            total,
            completed: 0,
            message: String::new(),
        }
    }

    /// Create a spinner-style progress bar (indeterminate).
    pub fn new_spinner() -> Self {
        let bar = indicatif::ProgressBar::new_spinner();
        bar.set_style(
            indicatif::ProgressStyle::default_spinner()
                .template("{spinner:.green} [{elapsed_precise}] {msg}")
                .unwrap(),
        );
        bar.enable_steady_tick(Duration::from_millis(100));
        Self {
            inner: Some(bar),
            total: 0,
            completed: 0,
            message: String::new(),
        }
    }

    /// Increment the progress bar by `delta` units.
    pub fn inc(&mut self, delta: u64) {
        self.completed += delta;
        if let Some(ref bar) = self.inner {
            bar.inc(delta);
        }
    }

    /// Set a message to display alongside the progress bar.
    pub fn set_message(&mut self, msg: impl Into<String>) {
        self.message = msg.into();
        if let Some(ref bar) = self.inner {
            bar.set_message(self.message.clone());
        }
    }

    /// Set the current position.
    pub fn set_position(&mut self, pos: u64) {
        self.completed = pos;
        if let Some(ref bar) = self.inner {
            bar.set_position(pos);
        }
    }

    /// Finish the progress bar.
    pub fn finish(&mut self) {
        if let Some(ref bar) = self.inner {
            bar.finish_with_message(self.message.clone());
        }
    }

    /// Finish the progress bar with a specific message.
    pub fn finish_with_message(&mut self, msg: impl Into<String>) {
        let msg = msg.into();
        self.message = msg.clone();
        if let Some(ref bar) = self.inner {
            bar.finish_with_message(msg);
        }
    }

    /// Abort the progress bar, indicating failure.
    pub fn abort(&mut self) {
        if let Some(ref bar) = self.inner {
            bar.abandon_with_message(format!("{} (aborted)", self.message));
        }
    }

    /// Return the total number of units.
    pub fn total(&self) -> u64 {
        self.total
    }

    /// Return the number of completed units.
    pub fn completed(&self) -> u64 {
        self.completed
    }

    /// Return the current message.
    pub fn message(&self) -> &str {
        &self.message
    }

    /// Return the current progress as a fraction (0.0 to 1.0).
    pub fn fraction(&self) -> f64 {
        if self.total == 0 {
            return 0.0;
        }
        self.completed as f64 / self.total as f64
    }
}

impl Drop for ProgressBar {
    fn drop(&mut self) {
        if let Some(ref bar) = self.inner {
            bar.finish_and_clear();
        }
    }
}

/// A manager for multiple progress bars displayed simultaneously.
pub struct MultiProgress {
    inner: indicatif::MultiProgress,
    bars: Vec<ProgressBar>,
}

impl MultiProgress {
    /// Create a new multi-progress manager.
    pub fn new() -> Self {
        Self {
            inner: indicatif::MultiProgress::new(),
            bars: Vec::new(),
        }
    }

    /// Add a progress bar to the multi-progress display.
    pub fn add(&mut self, mut bar: ProgressBar) -> usize {
        let idx = self.bars.len();
        if let Some(inner_bar) = bar.inner.take() {
            self.inner.add(inner_bar);
        }
        self.bars.push(bar);
        idx
    }

    /// Get a mutable reference to a progress bar by index.
    pub fn get_mut(&mut self, idx: usize) -> Option<&mut ProgressBar> {
        self.bars.get_mut(idx)
    }

    /// Clear all progress bars.
    pub fn clear(&mut self) {
        self.inner.clear().ok();
        self.bars.clear();
    }

    /// Join all progress bars (wait for them to finish).
    pub fn join(&self) {
    }

    /// Return the number of bars.
    pub fn len(&self) -> usize {
        self.bars.len()
    }

    /// Check if there are no bars.
    pub fn is_empty(&self) -> bool {
        self.bars.is_empty()
    }
}

/// Run a function `f` on each item, displaying a progress bar.
///
/// The progress bar advances as each item is processed.
pub fn with_progress<T>(
    items: Vec<T>,
    f: impl Fn(T) -> Result<(), String>,
) -> Result<(), ProgressError> {
    let total = items.len() as u64;
    let mut bar = ProgressBar::new(total);
    bar.set_message("Processing...");

    for item in items {
        if let Err(msg) = f(item) {
            bar.abort();
            return Err(ProgressError::Internal);
        }
        bar.inc(1);
    }

    bar.finish();
    Ok(())
}

/// Run an async function on each item with a progress bar.
pub async fn with_progress_async<T, Fut>(
    items: Vec<T>,
    f: impl Fn(T) -> Fut,
) -> Result<(), ProgressError>
where
    Fut: std::future::Future<Output = Result<(), String>>,
{
    let total = items.len() as u64;
    let mut bar = ProgressBar::new(total);
    bar.set_message("Processing...");

    for item in items {
        if let Err(msg) = f(item).await {
            bar.abort();
            return Err(ProgressError::Internal);
        }
        bar.inc(1);
    }

    bar.finish();
    Ok(())
}

/// A timer that tracks elapsed time for progress reporting.
#[derive(Debug, Clone)]
pub struct ProgressTimer {
    start: Instant,
}

impl ProgressTimer {
    /// Start a new timer.
    pub fn start() -> Self {
        Self {
            start: Instant::now(),
        }
    }

    /// Return the elapsed duration.
    pub fn elapsed(&self) -> Duration {
        self.start.elapsed()
    }

    /// Format the elapsed time.
    pub fn elapsed_formatted(&self) -> String {
        let secs = self.elapsed().as_secs();
        if secs < 60 {
            format!("{}s", secs)
        } else {
            format!("{}m {}s", secs / 60, secs % 60)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_progress_bar_new() {
        let bar = ProgressBar::new(100);
        assert_eq!(bar.total(), 100);
        assert_eq!(bar.completed(), 0);
        assert_eq!(bar.fraction(), 0.0);
    }

    #[test]
    fn test_progress_bar_inc() {
        let mut bar = ProgressBar::new(10);
        bar.inc(3);
        assert_eq!(bar.completed(), 3);
        assert!((bar.fraction() - 0.3).abs() < f64::EPSILON);
        bar.inc(7);
        assert_eq!(bar.completed(), 10);
        assert!((bar.fraction() - 1.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_progress_bar_message() {
        let mut bar = ProgressBar::new(5);
        bar.set_message("Downloading...");
        assert_eq!(bar.message(), "Downloading...");
    }

    #[test]
    fn test_progress_timer() {
        let timer = ProgressTimer::start();
        std::thread::sleep(Duration::from_millis(10));
        assert!(timer.elapsed() >= Duration::from_millis(10));
        let formatted = timer.elapsed_formatted();
        assert!(!formatted.is_empty());
    }

    #[test]
    fn test_multi_progress() {
        let mut mp = MultiProgress::new();
        assert!(mp.is_empty());
        mp.add(ProgressBar::new(10));
        assert_eq!(mp.len(), 1);
        mp.add(ProgressBar::new(20));
        assert_eq!(mp.len(), 2);
    }

    #[test]
    fn test_progress_bar_set_position() {
        let mut bar = ProgressBar::new(50);
        bar.set_position(25);
        assert_eq!(bar.completed(), 25);
        assert!((bar.fraction() - 0.5).abs() < f64::EPSILON);
    }
}
