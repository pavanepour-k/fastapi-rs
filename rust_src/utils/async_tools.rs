use pyo3::prelude::*;
use pyo3::types::{PyAny, PyCoroutine};
use std::future::Future;
use std::pin::Pin;
use std::task::{Context, Poll};
use tokio::runtime::Runtime;
use once_cell::sync::Lazy;

use pyo3::prelude::*;
use pyo3::types::{PyAny, PyCoroutine};
use std::future::Future;
use std::pin::Pin;
use std::task::{Context, Poll};
use tokio::runtime::Runtime;
use once_cell::sync::Lazy;

/// Global async runtime for FastAPI-RS
static ASYNC_RUNTIME: Lazy<Runtime> = Lazy::new(|| {
    tokio::runtime::Builder::new_multi_thread()
        .worker_threads(4)
        .enable_all()
        .build()
        .expect("Failed to create async runtime")
});

/// Run async Python coroutine in Rust
pub fn run_async_python(coro: &Bound<PyAny>) -> PyResult<Py<PyAny>> {
    Python::with_gil(|py| {
        let asyncio = py.import("asyncio")?;
        let loop_func = asyncio.getattr("get_event_loop")?;
        let event_loop = loop_func.call0()?;
        
        let run_until_complete = event_loop.getattr("run_until_complete")?;
        let result = run_until_complete.call1((coro,))?;
        
        Ok(result.into_py(py))
    })
}

/// Bridge for running Python async functions from Rust
pub async fn call_python_async<F, R>(func: F) -> PyResult<R>
where
    F: FnOnce(Python) -> PyResult<R> + Send + 'static,
    R: Send + 'static,
{
    tokio::task::spawn_blocking(move || {
        Python::with_gil(func)
    }).await.map_err(|e| {
        PyErr::new::<pyo3::exceptions::PyRuntimeError, _>(format!("Async task failed: {}", e))
    })?
}

/// Wrapper for Python coroutines that can be awaited in Rust
pub struct PyCoroutineWrapper {
    coro: Py<PyAny>,
}

impl PyCoroutineWrapper {
    pub fn new(coro: Py<PyAny>) -> Self {
        Self { coro }
    }
}

impl Future for PyCoroutineWrapper {
    type Output = PyResult<Py<PyAny>>;
    
    fn poll(self: Pin<&mut Self>, _cx: &mut Context<'_>) -> Poll<Self::Output> {
        // This is a simplified implementation
        // In a real implementation, you'd need proper async integration
        let result = Python::with_gil(|py| {
            let coro = self.coro.bind(py);
            run_async_python(coro)
        });
        
        Poll::Ready(result)
    }
}

/// Execute a blocking function in the async runtime
pub fn spawn_blocking<F, R>(func: F) -> impl Future<Output = Result<R, tokio::task::JoinError>>
where
    F: FnOnce() -> R + Send + 'static,
    R: Send + 'static,
{
    ASYNC_RUNTIME.spawn_blocking(func)
}

/// Execute an async function in the runtime
pub fn spawn_async<F>(future: F) -> tokio::task::JoinHandle<F::Output>
where
    F: Future + Send + 'static,
    F::Output: Send + 'static,
{
    ASYNC_RUNTIME.spawn(future)
}

/// Async context manager for Python integration
pub struct AsyncContext {
    runtime_handle: tokio::runtime::Handle,
}

impl AsyncContext {
    pub fn new() -> Self {
        Self {
            runtime_handle: ASYNC_RUNTIME.handle().clone(),
        }
    }
    
    pub fn block_on<F>(&self, future: F) -> F::Output
    where
        F: Future,
    {
        self.runtime_handle.block_on(future)
    }
    
    pub fn spawn<F>(&self, future: F) -> tokio::task::JoinHandle<F::Output>
    where
        F: Future + Send + 'static,
        F::Output: Send + 'static,
    {
        self.runtime_handle.spawn(future)
    }
}

/// Async timeout wrapper
pub async fn with_timeout<F, T>(
    duration: std::time::Duration,
    future: F,
) -> Result<T, tokio::time::error::Elapsed>
where
    F: Future<Output = T>,
{
    tokio::time::timeout(duration, future).await
}

/// Async batch processor for handling multiple operations
pub async fn batch_process<T, R, F, Fut>(
    items: Vec<T>,
    processor: F,
    batch_size: usize,
) -> Vec<Result<R, Box<dyn std::error::Error + Send + Sync>>>
where
    T: Send + 'static,
    R: Send + 'static,
    F: Fn(T) -> Fut + Send + Sync + Clone + 'static,
    Fut: Future<Output = Result<R, Box<dyn std::error::Error + Send + Sync>>> + Send + 'static,
{
    let mut results = Vec::new();
    
    for chunk in items.chunks(batch_size) {
        let tasks: Vec<_> = chunk.iter()
            .cloned()
            .map(|item| {
                let proc = processor.clone();
                tokio::spawn(async move { proc(item).await })
            })
            .collect();
        
        for task in tasks {
            match task.await {
                Ok(result) => results.push(result),
                Err(e) => results.push(Err(Box::new(e) as Box<dyn std::error::Error + Send + Sync>)),
            }
        }
    }
    
    results
}

/// Async signal handling for graceful shutdown
pub struct AsyncSignalHandler {
    shutdown_tx: tokio::sync::broadcast::Sender<()>,
    shutdown_rx: tokio::sync::broadcast::Receiver<()>,
}

impl AsyncSignalHandler {
    pub fn new() -> Self {
        let (shutdown_tx, shutdown_rx) = tokio::sync::broadcast::channel(1);
        
        Self {
            shutdown_tx,
            shutdown_rx,
        }
    }
    
    pub async fn wait_for_shutdown(&mut self) {
        let _ = self.shutdown_rx.recv().await;
    }
    
    pub fn trigger_shutdown(&self) {
        let _ = self.shutdown_tx.send(());
    }
    
    pub fn subscribe(&self) -> tokio::sync::broadcast::Receiver<()> {
        self.shutdown_tx.subscribe()
    }
}

/// Async retry mechanism
pub async fn retry_async<F, Fut, T, E>(
    mut operation: F,
    max_attempts: u32,
    delay: std::time::Duration,
) -> Result<T, E>
where
    F: FnMut() -> Fut,
    Fut: Future<Output = Result<T, E>>,
    E: std::fmt::Debug,
{
    let mut attempts = 0;
    
    loop {
        attempts += 1;
        
        match operation().await {
            Ok(result) => return Ok(result),
            Err(e) if attempts >= max_attempts => return Err(e),
            Err(_) => {
                tokio::time::sleep(delay).await;
            }
        }
    }
}

/// Async rate limiter
pub struct AsyncRateLimiter {
    semaphore: tokio::sync::Semaphore,
    reset_interval: std::time::Duration,
}

impl AsyncRateLimiter {
    pub fn new(max_concurrent: usize, reset_interval: std::time::Duration) -> Self {
        Self {
            semaphore: tokio::sync::Semaphore::new(max_concurrent),
            reset_interval,
        }
    }
    
    pub async fn acquire(&self) -> tokio::sync::SemaphorePermit<'_> {
        self.semaphore.acquire().await.expect("Semaphore closed")
    }
    
    pub async fn try_acquire(&self) -> Option<tokio::sync::SemaphorePermit<'_>> {
        self.semaphore.try_acquire().ok()
    }
}

/// Async worker pool for background tasks
pub struct AsyncWorkerPool {
    task_tx: tokio::sync::mpsc::UnboundedSender<BoxedAsyncTask>,
    _worker_handles: Vec<tokio::task::JoinHandle<()>>,
}

type BoxedAsyncTask = Box<dyn FnOnce() -> Pin<Box<dyn Future<Output = ()> + Send>> + Send>;

impl AsyncWorkerPool {
    pub fn new(worker_count: usize) -> Self {
        let (task_tx, mut task_rx) = tokio::sync::mpsc::unbounded_channel::<BoxedAsyncTask>();
        
        let mut worker_handles = Vec::new();
        
        for _ in 0..worker_count {
            let mut rx = task_rx.clone();
            let handle = tokio::spawn(async move {
                while let Some(task) = rx.recv().await {
                    let future = task();
                    future.await;
                }
            });
            worker_handles.push(handle);
        }
        
        // Close the original receiver
        task_rx.close();
        
        Self {
            task_tx,
            _worker_handles: worker_handles,
        }
    }
    
    pub fn spawn<F, Fut>(&self, task: F) -> Result<(), tokio::sync::mpsc::error::SendError<BoxedAsyncTask>>
    where
        F: FnOnce() -> Fut + Send + 'static,
        Fut: Future<Output = ()> + Send + 'static,
    {
        let boxed_task = Box::new(move || Box::pin(task()) as Pin<Box<dyn Future<Output = ()> + Send>>);
        self.task_tx.send(boxed_task)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration;
    
    #[tokio::test]
    async fn test_async_context() {
        let ctx = AsyncContext::new();
        
        let result = ctx.block_on(async {
            tokio::time::sleep(Duration::from_millis(10)).await;
            42
        });
        
        assert_eq!(result, 42);
    }
    
    #[tokio::test]
    async fn test_with_timeout() {
        let result = with_timeout(
            Duration::from_millis(100),
            async {
                tokio::time::sleep(Duration::from_millis(50)).await;
                "success"
            }
        ).await;
        
        assert_eq!(result.unwrap(), "success");
        
        let timeout_result = with_timeout(
            Duration::from_millis(50),
            async {
                tokio::time::sleep(Duration::from_millis(100)).await;
                "success"
            }
        ).await;
        
        assert!(timeout_result.is_err());
    }
    
    #[tokio::test]
    async fn test_retry_async() {
        let mut counter = 0;
        
        let result = retry_async(
            || {
                counter += 1;
                async move {
                    if counter < 3 {
                        Err("not ready")
                    } else {
                        Ok("success")
                    }
                }
            },
            5,
            Duration::from_millis(10),
        ).await;
        
        assert_eq!(result.unwrap(), "success");
        assert_eq!(counter, 3);
    }
    
    #[tokio::test]
    async fn test_rate_limiter() {
        let limiter = AsyncRateLimiter::new(2, Duration::from_secs(1));
        
        let permit1 = limiter.try_acquire();
        let permit2 = limiter.try_acquire();
        let permit3 = limiter.try_acquire();
        
        assert!(permit1.is_some());
        assert!(permit2.is_some());
        assert!(permit3.is_none());
        
        drop(permit1);
        
        let permit4 = limiter.try_acquire();
        assert!(permit4.is_some());
    }
}