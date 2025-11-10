//! # Tokio Tutorial Patterns and Use Cases
//!
//! This library contains reusable patterns and utilities for working with Tokio.
//! Each module corresponds to a section in the tutorial documentation.

pub mod basic_operations {
    //! Basic Tokio runtime operations and configurations

    use tokio::runtime::Runtime;

    /// Creates a new multi-threaded Tokio runtime
    pub fn create_runtime() -> Runtime {
        Runtime::new().expect("Failed to create runtime")
    }

    /// Creates a single-threaded Tokio runtime
    pub fn create_current_thread_runtime() -> Runtime {
        tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .expect("Failed to create current thread runtime")
    }

    /// Creates a multi-threaded runtime with custom worker threads
    pub fn create_runtime_with_threads(num_threads: usize) -> Runtime {
        tokio::runtime::Builder::new_multi_thread()
            .worker_threads(num_threads)
            .enable_all()
            .build()
            .expect("Failed to create runtime with custom threads")
    }
}

pub mod spawning {
    //! Task spawning patterns and utilities

    use std::sync::Arc;
    use tokio::task::JoinHandle;

    /// Spawns multiple tasks that share immutable data using Arc
    pub fn spawn_with_shared_data<T>(data: Arc<T>, count: usize) -> Vec<JoinHandle<()>>
    where
        T: Send + Sync + 'static + std::fmt::Debug,
    {
        (0..count)
            .map(|i| {
                let data_clone = Arc::clone(&data);
                tokio::spawn(async move {
                    println!("Task {} sees: {:?}", i, data_clone);
                })
            })
            .collect()
    }

    /// Waits for all tasks to complete
    pub async fn wait_for_tasks(handles: Vec<JoinHandle<()>>) {
        for handle in handles {
            let _ = handle.await;
        }
    }

    /// Demonstrates task cancellation
    pub async fn cancellable_task() -> JoinHandle<()> {
        tokio::spawn(async {
            loop {
                tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
                println!("Task running...");
            }
        })
    }
}

pub mod shared_state {
    //! Patterns for sharing state across async tasks

    use std::sync::Arc;
    use tokio::sync::{Mutex, RwLock, Semaphore, Barrier, Notify};

    /// A thread-safe counter using Arc and Mutex
    #[derive(Clone)]
    pub struct Counter {
        inner: Arc<Mutex<i32>>,
    }

    impl Counter {
        pub fn new(initial: i32) -> Self {
            Self {
                inner: Arc::new(Mutex::new(initial)),
            }
        }

        pub async fn increment(&self) {
            let mut count = self.inner.lock().await;
            *count += 1;
        }

        pub async fn get(&self) -> i32 {
            *self.inner.lock().await
        }
    }

    /// A read-write locked data structure
    pub struct SharedData<T> {
        inner: Arc<RwLock<T>>,
    }

    impl<T> SharedData<T> {
        pub fn new(data: T) -> Self {
            Self {
                inner: Arc::new(RwLock::new(data)),
            }
        }

        pub async fn read(&self) -> tokio::sync::RwLockReadGuard<T> {
            self.inner.read().await
        }

        pub async fn write(&self) -> tokio::sync::RwLockWriteGuard<T> {
            self.inner.write().await
        }
    }

    impl<T> Clone for SharedData<T> {
        fn clone(&self) -> Self {
            Self {
                inner: Arc::clone(&self.inner),
            }
        }
    }

    /// Creates a semaphore for limiting concurrent access
    pub fn create_semaphore(permits: usize) -> Arc<Semaphore> {
        Arc::new(Semaphore::new(permits))
    }

    /// Creates a barrier for synchronizing multiple tasks
    pub fn create_barrier(count: usize) -> Arc<Barrier> {
        Arc::new(Barrier::new(count))
    }

    /// Creates a notify primitive for signaling between tasks
    pub fn create_notify() -> Arc<Notify> {
        Arc::new(Notify::new())
    }
}

pub mod channels {
    //! Channel patterns for task communication

    use tokio::sync::{mpsc, oneshot, broadcast, watch};

    /// Creates an MPSC channel with the specified buffer size
    pub fn create_mpsc<T>(buffer: usize) -> (mpsc::Sender<T>, mpsc::Receiver<T>) {
        mpsc::channel(buffer)
    }

    /// Creates an unbounded MPSC channel
    pub fn create_unbounded_mpsc<T>() -> (mpsc::UnboundedSender<T>, mpsc::UnboundedReceiver<T>) {
        mpsc::unbounded_channel()
    }

    /// Creates a oneshot channel
    pub fn create_oneshot<T>() -> (oneshot::Sender<T>, oneshot::Receiver<T>) {
        oneshot::channel()
    }

    /// Creates a broadcast channel
    pub fn create_broadcast<T: Clone>(capacity: usize) -> broadcast::Sender<T> {
        let (tx, _rx) = broadcast::channel(capacity);
        tx
    }

    /// Creates a watch channel
    pub fn create_watch<T: Clone>(initial: T) -> (watch::Sender<T>, watch::Receiver<T>) {
        watch::channel(initial)
    }

    /// Request-response pattern using oneshot channels
    pub struct RequestHandler<Req, Resp> {
        tx: mpsc::Sender<(Req, oneshot::Sender<Resp>)>,
    }

    impl<Req, Resp> RequestHandler<Req, Resp>
    where
        Req: Send + 'static,
        Resp: Send + 'static,
    {
        pub fn new<F, Fut>(mut handler: F) -> Self
        where
            F: FnMut(Req) -> Fut + Send + 'static,
            Fut: std::future::Future<Output = Resp> + Send,
        {
            let (tx, mut rx) = mpsc::channel::<(Req, oneshot::Sender<Resp>)>(32);

            tokio::spawn(async move {
                while let Some((req, response_tx)) = rx.recv().await {
                    let resp = handler(req).await;
                    let _ = response_tx.send(resp);
                }
            });

            Self { tx }
        }

        pub async fn request(&self, req: Req) -> Result<Resp, oneshot::error::RecvError> {
            let (resp_tx, resp_rx) = oneshot::channel();
            let _ = self.tx.send((req, resp_tx)).await;
            resp_rx.await
        }
    }

    impl<Req, Resp> Clone for RequestHandler<Req, Resp> {
        fn clone(&self) -> Self {
            Self {
                tx: self.tx.clone(),
            }
        }
    }
}

pub mod io {
    //! Async I/O patterns and utilities

    use tokio::io::{AsyncReadExt, AsyncWriteExt};
    use std::path::Path;

    /// Asynchronously reads the entire contents of a file
    pub async fn read_file<P: AsRef<Path>>(path: P) -> std::io::Result<Vec<u8>> {
        let mut file = tokio::fs::File::open(path).await?;
        let mut contents = Vec::new();
        file.read_to_end(&mut contents).await?;
        Ok(contents)
    }

    /// Asynchronously writes data to a file
    pub async fn write_file<P: AsRef<Path>>(path: P, contents: &[u8]) -> std::io::Result<()> {
        let mut file = tokio::fs::File::create(path).await?;
        file.write_all(contents).await?;
        Ok(())
    }

    /// Copies a file asynchronously
    pub async fn copy_file<P: AsRef<Path>>(from: P, to: P) -> std::io::Result<u64> {
        tokio::fs::copy(from, to).await
    }

    /// Creates a TCP echo server on the given address
    pub async fn tcp_echo_server(addr: &str) -> std::io::Result<()> {
        use tokio::net::TcpListener;

        let listener = TcpListener::bind(addr).await?;
        println!("Echo server listening on: {}", addr);

        loop {
            let (mut socket, _) = listener.accept().await?;

            tokio::spawn(async move {
                let mut buf = vec![0; 1024];

                loop {
                    match socket.read(&mut buf).await {
                        Ok(0) => return,
                        Ok(n) => {
                            if socket.write_all(&buf[0..n]).await.is_err() {
                                return;
                            }
                        }
                        Err(_) => return,
                    }
                }
            });
        }
    }
}

pub mod select {
    //! Patterns using tokio::select! for concurrent operations

    use tokio::time::{sleep, Duration};

    /// Demonstrates basic select pattern
    pub async fn select_with_timeout<T>(
        future: impl std::future::Future<Output = T>,
        timeout: Duration,
    ) -> Result<T, ()> {
        tokio::select! {
            result = future => Ok(result),
            _ = sleep(timeout) => Err(()),
        }
    }

    /// Graceful shutdown pattern
    pub async fn graceful_shutdown<F, Fut>(
        work: F,
        mut shutdown_rx: tokio::sync::mpsc::Receiver<()>,
    ) where
        F: Fn() -> Fut,
        Fut: std::future::Future<Output = ()>,
    {
        let mut interval = tokio::time::interval(Duration::from_millis(100));

        loop {
            tokio::select! {
                _ = interval.tick() => {
                    work().await;
                }
                _ = shutdown_rx.recv() => {
                    println!("Shutdown signal received");
                    break;
                }
            }
        }
    }
}

pub mod streams {
    //! Stream processing patterns and utilities

    use tokio_stream::{Stream, StreamExt};
    use std::pin::Pin;
    use std::task::{Context, Poll};

    /// A custom Fibonacci stream
    pub struct FibonacciStream {
        curr: u64,
        next: u64,
    }

    impl FibonacciStream {
        pub fn new() -> Self {
            Self { curr: 0, next: 1 }
        }
    }

    impl Default for FibonacciStream {
        fn default() -> Self {
            Self::new()
        }
    }

    impl Stream for FibonacciStream {
        type Item = u64;

        fn poll_next(mut self: Pin<&mut Self>, _cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
            let current = self.curr;
            let next = self.next;

            self.curr = next;
            self.next = current + next;

            Poll::Ready(Some(current))
        }
    }

    /// Takes the first n items from a stream
    pub async fn take_n<S>(mut stream: S, n: usize) -> Vec<S::Item>
    where
        S: Stream + Unpin,
    {
        let mut items = Vec::new();
        let mut count = 0;

        while count < n {
            if let Some(item) = stream.next().await {
                items.push(item);
                count += 1;
            } else {
                break;
            }
        }

        items
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_counter() {
        let counter = shared_state::Counter::new(0);

        let mut handles = vec![];
        for _ in 0..10 {
            let counter = counter.clone();
            handles.push(tokio::spawn(async move {
                counter.increment().await;
            }));
        }

        for handle in handles {
            handle.await.unwrap();
        }

        assert_eq!(counter.get().await, 10);
    }

    #[tokio::test]
    async fn test_request_handler() {
        let handler = channels::RequestHandler::new(|x: i32| async move { x * 2 });

        let result = handler.request(5).await.unwrap();
        assert_eq!(result, 10);
    }

    #[tokio::test]
    async fn test_fibonacci_stream() {
        let stream = streams::FibonacciStream::new();
        let items = streams::take_n(stream, 10).await;

        assert_eq!(items, vec![0, 1, 1, 2, 3, 5, 8, 13, 21, 34]);
    }
}
