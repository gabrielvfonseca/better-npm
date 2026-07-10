//! Async and parallel concurrency primitives for Dash workflows.
//!
//! Provides helpers for bounded parallelism, task groups,
//! rate limiting, and parallel map/filter patterns.

use futures::future::join_all;
use parking_lot::Mutex;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::Semaphore;
use tokio::task::JoinHandle;

/// Error type for concurrency operations.
#[derive(Debug, thiserror::Error)]
pub enum ConcurrencyError {
    #[error("task panicked: {0}")]
    TaskPanic(String),
    #[error("join error: {0}")]
    Join(#[from] tokio::task::JoinError),
}

/// Run a closure `f` on each item in `items` in parallel, with bounded concurrency.
///
/// Returns results in the same order as the input.
pub async fn parallel_map<T, U, F>(items: Vec<T>, f: F, concurrency: usize) -> Vec<U>
where
    T: Send + 'static,
    U: Send + 'static,
    F: Fn(T) -> U + Send + Sync + 'static,
{
    let semaphore = Arc::new(Semaphore::new(concurrency));
    let results: Arc<Mutex<Vec<(usize, U)>>> = Arc::new(Mutex::new(Vec::with_capacity(items.len())));
    let f = Arc::new(f);

    let mut handles = Vec::with_capacity(items.len());
    for (i, item) in items.into_iter().enumerate() {
        let sem = Arc::clone(&semaphore);
        let results = Arc::clone(&results);
        let f = Arc::clone(&f);
        handles.push(tokio::spawn(async move {
            let _permit = sem.acquire().await.expect("semaphore closed");
            let output = f(item);
            results.lock().push((i, output));
        }));
    }

    for h in handles {
        let _ = h.await;
    }

    let mut sorted = results.lock();
    sorted.sort_by_key(|(i, _)| *i);
    sorted.drain(..).map(|(_, v)| v).collect()
}

/// Run a predicate `f` on each item in parallel, returning only items for which
/// the predicate returns `true`, with bounded concurrency.
pub async fn parallel_filter<T, F>(items: Vec<T>, f: F, concurrency: usize) -> Vec<T>
where
    T: Send + 'static,
    F: Fn(&T) -> bool + Send + Sync + 'static,
{
    let semaphore = Arc::new(Semaphore::new(concurrency));
    let results: Arc<Mutex<Vec<(usize, T)>>> = Arc::new(Mutex::new(Vec::new()));
    let f = Arc::new(f);

    let mut handles = Vec::with_capacity(items.len());
    for (i, item) in items.into_iter().enumerate() {
        let sem = Arc::clone(&semaphore);
        let results = Arc::clone(&results);
        let f = Arc::clone(&f);
        handles.push(tokio::spawn(async move {
            let _permit = sem.acquire().await.expect("semaphore closed");
            if f(&item) {
                results.lock().push((i, item));
            }
        }));
    }

    for h in handles {
        let _ = h.await;
    }

    let mut sorted = results.lock();
    sorted.sort_by_key(|(i, _)| *i);
    sorted.drain(..).map(|(_, v)| v).collect()
}

/// A group of spawned async tasks with bounded concurrency.
pub struct TaskGroup {
    handles: Vec<JoinHandle<()>>,
    semaphore: Arc<Semaphore>,
}

impl TaskGroup {
    /// Create a new task group with the given concurrency limit.
    pub fn new(concurrency: usize) -> Self {
        Self {
            handles: Vec::new(),
            semaphore: Arc::new(Semaphore::new(concurrency)),
        }
    }

    /// Spawn a new task in the group. Blocks if at capacity.
    pub fn spawn<F>(&mut self, f: F)
    where
        F: std::future::Future<Output = ()> + Send + 'static,
    {
        let sem = Arc::clone(&self.semaphore);
        let handle = tokio::spawn(async move {
            let _permit = sem.acquire().await.expect("semaphore closed");
            f.await;
        });
        self.handles.push(handle);
    }

    /// Wait for all spawned tasks to complete.
    pub async fn wait_all(self) {
        join_all(self.handles).await;
    }

    /// Return the concurrency limit.
    pub fn concurrency(&self) -> usize {
        self.semaphore.available_permits() + self.handles.len()
    }
}

/// A rate-limiter that limits the number of operations over a time interval.
pub struct Throttle {
    semaphore: Arc<Semaphore>,
    interval: Duration,
}

impl Throttle {
    /// Create a new throttle with `permits` operations per `interval`.
    pub fn new(permits: usize, interval: Duration) -> Self {
        Self {
            semaphore: Arc::new(Semaphore::new(permits)),
            interval,
        }
    }

    /// Acquire a permit, waiting if necessary.
    pub async fn acquire(&self) {
        let _permit = self.semaphore.acquire().await.expect("semaphore closed");
        tokio::time::sleep(self.interval).await;
        drop(_permit);
    }

    /// Acquire a permit with a timeout.
    pub async fn try_acquire(&self, timeout: Duration) -> Result<(), ConcurrencyError> {
        let _permit = tokio::time::timeout(timeout, self.semaphore.acquire())
            .await
            .map_err(|_| ConcurrencyError::TaskPanic("timeout waiting for throttle permit".into()))?
            .map_err(|_| ConcurrencyError::TaskPanic("semaphore closed".into()))?;

        tokio::time::sleep(self.interval).await;
        drop(_permit);

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_parallel_map() {
        let items = vec![1, 2, 3, 4, 5];
        let results = parallel_map(items, |x| x * 2, 3).await;
        assert_eq!(results, vec![2, 4, 6, 8, 10]);
    }

    #[tokio::test]
    async fn test_parallel_filter() {
        let items = vec![1, 2, 3, 4, 5, 6];
        let results = parallel_filter(items, |x| x % 2 == 0, 2).await;
        assert_eq!(results, vec![2, 4, 6]);
    }

    #[tokio::test]
    async fn test_task_group() {
        let mut group = TaskGroup::new(2);
        let counter = Arc::new(std::sync::atomic::AtomicUsize::new(0));
        let c = Arc::clone(&counter);
        group.spawn(async move {
            c.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
        });
        let c = Arc::clone(&counter);
        group.spawn(async move {
            c.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
        });
        group.wait_all().await;
        assert_eq!(counter.load(std::sync::atomic::Ordering::SeqCst), 2);
    }

    #[tokio::test]
    async fn test_throttle_acquire() {
        let throttle = Throttle::new(5, Duration::from_millis(10));
        // Should be able to acquire 5 permits quickly
        for _ in 0..5 {
            throttle.acquire().await;
        }
    }
}
