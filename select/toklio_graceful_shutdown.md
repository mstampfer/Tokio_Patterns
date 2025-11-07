# Graceful Shutdown Pattern with `tokio::select!`

## Overview

This code demonstrates a **graceful shutdown pattern** - one of the most important patterns in async Rust programming. It shows how to build a worker task that continuously performs work while remaining responsive to shutdown signals, allowing for proper cleanup before terminating.

## The Graceful Shutdown Pattern

A graceful shutdown has three key requirements:

1. **Responsiveness**: The task must detect shutdown signals quickly, not wait for long-running operations to complete
2. **Completion**: Current work should finish before shutting down (no abrupt cancellation)
3. **Cleanup**: Resources should be properly released and final operations performed

## How This Code Implements Graceful Shutdown

### Architecture Overview

```
Main Task                          Worker Task
    |                                  |
    |                                  ├─> Periodic work (every 100ms)
    |                                  |
    |                                  ├─> Listen for shutdown
    |                                  |
    ├─> Wait 350ms                     ├─> Work tick 1 (100ms)
    |                                  ├─> Work tick 2 (200ms)
    |                                  ├─> Work tick 3 (300ms)
    |                                  |
    ├─> Send shutdown signal ────────>├─> Receive shutdown
    |                                  ├─> Cleanup
    |                                  ├─> Exit gracefully
    |                                  |
    ├─> Wait for worker to finish <────┘
    |
    └─> Complete
```

## Code Breakdown

### Step 1: Create Shutdown Channel

```rust
let (shutdown_tx, mut shutdown_rx) = mpsc::channel::<()>(1);
```

An MPSC channel is created specifically for shutdown signaling:
- **Sender** (`shutdown_tx`): Held by the main task to trigger shutdown
- **Receiver** (`shutdown_rx`): Held by the worker to listen for shutdown
- **Buffer size 1**: Only one shutdown signal needed
- **Message type `()`**: No data needed, just a signal

### Step 2: Spawn the Worker Task

```rust
let worker = tokio::spawn(async move {
    let mut tick = interval(Duration::from_millis(100));
    let mut counter = 0;
    
    loop {
        tokio::select! {
            _ = tick.tick() => {
                counter += 1;
                println!("Working... {}", counter);
            }
            _ = shutdown_rx.recv() => {
                println!("Shutdown signal received");
                // Perform cleanup
                break;
            }
        }
    }
    
    println!("Worker shut down gracefully");
});
```

**Key components:**

1. **`interval(Duration::from_millis(100))`**: Creates a timer that fires every 100ms
2. **`tick.tick()`**: Waits for the next interval tick
3. **`select!` with two branches**:
   - **Work branch**: Executes periodic work when the interval ticks
   - **Shutdown branch**: Listens for shutdown signal
4. **`break` on shutdown**: Exits the loop when shutdown is received
5. **Cleanup message**: Code after the loop runs during graceful shutdown

### Step 3: Let the Worker Run

```rust
sleep(Duration::from_millis(350)).await;
```

The main task sleeps for 350ms, allowing the worker to perform several work cycles:
- Tick 1 at ~100ms
- Tick 2 at ~200ms  
- Tick 3 at ~300ms

### Step 4: Send Shutdown Signal

```rust
shutdown_tx.send(()).await.unwrap();
```

After 350ms, the main task sends a shutdown signal through the channel. This immediately makes the `shutdown_rx.recv()` branch in the worker's `select!` ready.

### Step 5: Wait for Graceful Completion

```rust
worker.await.unwrap();
```

The main task waits for the worker to fully complete, ensuring:
- The worker receives the shutdown signal
- Any cleanup code runs
- The worker task exits cleanly

## Execution Timeline

```
Time    Event
----    -----
0ms     Worker spawned, starts listening
        Main task starts sleeping
        
~100ms  interval.tick() fires
        "Working... 1" printed
        select! waits again
        
~200ms  interval.tick() fires
        "Working... 2" printed
        select! waits again
        
~300ms  interval.tick() fires
        "Working... 3" printed
        select! waits again
        
350ms   Main task wakes up
        Sends shutdown signal through channel
        
~350ms  shutdown_rx.recv() receives signal (becomes ready)
        select! chooses shutdown branch
        "Shutdown signal received" printed
        Loop breaks
        "Worker shut down gracefully" printed
        Worker task completes
        
~350ms  Main task's worker.await completes
        Program exits
```

## Why `select!` is Essential for Graceful Shutdown

### Without `select!` (Problematic)

```rust
// ❌ BAD: Can't respond to shutdown until interval completes
loop {
    tick.tick().await;
    counter += 1;
    println!("Working... {}", counter);
    
    // Check shutdown only between work cycles
    if let Ok(_) = shutdown_rx.try_recv() {
        break;
    }
}
```

**Problems:**
- Shutdown only checked after `tick.tick().await` completes
- If work is long-running, shutdown is delayed
- No true concurrency between work and shutdown listening

### With `select!` (Correct)

```rust
// ✅ GOOD: Can respond to shutdown at any time
loop {
    tokio::select! {
        _ = tick.tick() => {
            counter += 1;
            println!("Working... {}", counter);
        }
        _ = shutdown_rx.recv() => {
            println!("Shutdown signal received");
            break;
        }
    }
}
```

**Benefits:**
- Both branches are polled concurrently
- Shutdown signal detected immediately when sent
- Worker remains responsive even during ticks
- Clean separation of work and shutdown logic

## Complete Code

```rust
use tokio::sync::mpsc;
use tokio::time::{sleep, Duration, interval};

#[tokio::main]
async fn main() {
    let (shutdown_tx, mut shutdown_rx) = mpsc::channel::<()>(1);
    
    let worker = tokio::spawn(async move {
        let mut tick = interval(Duration::from_millis(100));
        let mut counter = 0;
        
        loop {
            tokio::select! {
                _ = tick.tick() => {
                    counter += 1;
                    println!("Working... {}", counter);
                }
                _ = shutdown_rx.recv() => {
                    println!("Shutdown signal received");
                    // Perform cleanup
                    break;
                }
            }
        }
        
        println!("Worker shut down gracefully");
    });
    
    // Let it work for a bit
    sleep(Duration::from_millis(350)).await;
    
    // Send shutdown signal
    shutdown_tx.send(()).await.unwrap();
    
    worker.await.unwrap();
}
```

## Expected Output

```
Working... 1
Working... 2
Working... 3
Shutdown signal received
Worker shut down gracefully
```

The worker completes 3-4 work cycles before receiving the shutdown signal and exiting gracefully.

## Advanced Pattern: Multiple Workers with Broadcast

For scenarios with multiple workers that all need to shut down, use a broadcast channel:

```rust
use tokio::sync::broadcast;
use tokio::time::{sleep, Duration, interval};

#[tokio::main]
async fn main() {
    let (shutdown_tx, _) = broadcast::channel::<()>(1);
    
    // Spawn multiple workers
    let mut workers = vec![];
    
    for id in 1..=3 {
        let mut shutdown_rx = shutdown_tx.subscribe();
        
        let worker = tokio::spawn(async move {
            let mut tick = interval(Duration::from_millis(100));
            let mut counter = 0;
            
            loop {
                tokio::select! {
                    _ = tick.tick() => {
                        counter += 1;
                        println!("Worker {} - tick {}", id, counter);
                    }
                    _ = shutdown_rx.recv() => {
                        println!("Worker {} shutting down", id);
                        break;
                    }
                }
            }
            
            println!("Worker {} shut down gracefully", id);
        });
        
        workers.push(worker);
    }
    
    // Let them work
    sleep(Duration::from_millis(350)).await;
    
    // Broadcast shutdown to all workers
    println!("Broadcasting shutdown signal");
    shutdown_tx.send(()).unwrap();
    
    // Wait for all workers to complete
    for worker in workers {
        worker.await.unwrap();
    }
    
    println!("All workers shut down");
}
```

### Output:
```
Worker 1 - tick 1
Worker 2 - tick 1
Worker 3 - tick 1
Worker 1 - tick 2
Worker 2 - tick 2
Worker 3 - tick 2
Worker 1 - tick 3
Worker 2 - tick 3
Worker 3 - tick 3
Broadcasting shutdown signal
Worker 1 shutting down
Worker 2 shutting down
Worker 3 shutting down
Worker 1 shut down gracefully
Worker 2 shut down gracefully
Worker 3 shut down gracefully
All workers shut down
```

## Pattern Variations

### 1. With Cleanup Logic

```rust
loop {
    tokio::select! {
        _ = tick.tick() => {
            // Do work
        }
        _ = shutdown_rx.recv() => {
            println!("Shutting down...");
            break;
        }
    }
}

// Cleanup happens here
println!("Closing connections...");
db_connection.close().await;
println!("Flushing logs...");
log_buffer.flush().await;
println!("Cleanup complete");
```

### 2. With Biased Priority (Check Shutdown First)

```rust
loop {
    tokio::select! {
        biased;  // Check shutdown before accepting new work
        
        _ = shutdown_rx.recv() => {
            println!("Shutdown signal received");
            break;
        }
        _ = tick.tick() => {
            counter += 1;
            println!("Working... {}", counter);
        }
    }
}
```

This ensures shutdown is checked before processing new work items.

### 3. With Timeout (Force Shutdown After Grace Period)

```rust
use tokio::time::timeout;

// Try to send shutdown
shutdown_tx.send(()).await.unwrap();

// Wait for graceful shutdown with timeout
match timeout(Duration::from_secs(5), worker).await {
    Ok(Ok(_)) => println!("Worker shut down gracefully"),
    Ok(Err(e)) => println!("Worker panicked: {}", e),
    Err(_) => println!("Worker didn't shut down in time, abandoning it"),
}
```

### 4. Signal Handler Integration (Real-World)

```rust
use tokio::signal;

#[tokio::main]
async fn main() {
    let (shutdown_tx, mut shutdown_rx) = mpsc::channel::<()>(1);
    
    // Spawn worker
    let worker = tokio::spawn(async move {
        let mut tick = interval(Duration::from_millis(100));
        
        loop {
            tokio::select! {
                _ = tick.tick() => {
                    println!("Working...");
                }
                _ = shutdown_rx.recv() => {
                    println!("Graceful shutdown initiated");
                    break;
                }
            }
        }
        
        println!("Worker finished");
    });
    
    // Wait for Ctrl+C
    tokio::select! {
        _ = signal::ctrl_c() => {
            println!("\nReceived Ctrl+C, initiating shutdown...");
        }
    }
    
    // Send shutdown signal
    shutdown_tx.send(()).await.unwrap();
    
    // Wait for worker
    worker.await.unwrap();
    
    println!("Application shut down cleanly");
}
```

## Best Practices

### 1. Always Use Channels for Shutdown Signals

```rust
// ✅ Good: Explicit shutdown channel
let (shutdown_tx, mut shutdown_rx) = mpsc::channel::<()>(1);

// ❌ Bad: Using atomic bools
let shutdown = Arc::new(AtomicBool::new(false));
// Requires polling, not async-friendly
```

### 2. Separate Work from Shutdown Logic

```rust
tokio::select! {
    // Clear separation of concerns
    result = do_work() => { handle_work(result); }
    _ = shutdown_rx.recv() => { break; }
}
```

### 3. Always Wait for Workers to Complete

```rust
// ✅ Good: Wait for graceful completion
worker.await.unwrap();

// ❌ Bad: Drop worker handle without waiting
drop(worker);  // Task keeps running but we can't track it!
```

### 4. Use Broadcast for Multiple Workers

```rust
// For 1 worker: mpsc
let (tx, rx) = mpsc::channel(1);

// For N workers: broadcast
let (tx, _) = broadcast::channel(1);
let rx1 = tx.subscribe();
let rx2 = tx.subscribe();
```

### 5. Consider Timeouts for Forced Shutdown

Always have a backup plan if graceful shutdown takes too long:

```rust
if timeout(Duration::from_secs(30), worker).await.is_err() {
    eprintln!("Worker didn't shut down, forcing termination");
    // Escalate shutdown
}
```

## Common Pitfalls

### Pitfall 1: Not Handling Receiver Drop

```rust
// Worker might exit if receiver is dropped
_ = shutdown_rx.recv() => {
    // Returns None if sender dropped
    break;
}
```

**Solution**: Handle `None` case explicitly:
```rust
result = shutdown_rx.recv() => {
    match result {
        Some(_) => println!("Graceful shutdown"),
        None => println!("Shutdown channel closed"),
    }
    break;
}
```

### Pitfall 2: Forgetting to Break the Loop

```rust
// ❌ Bad: Continues running after shutdown!
_ = shutdown_rx.recv() => {
    println!("Shutdown received");
    // Missing break!
}
```

### Pitfall 3: Not Awaiting the Worker

```rust
// ❌ Bad: Main exits, worker gets dropped
shutdown_tx.send(()).await.unwrap();
// Program exits, worker might not finish cleanup!
```

**Solution**: Always await:
```rust
shutdown_tx.send(()).await.unwrap();
worker.await.unwrap();  // ✅ Wait for completion
```

## Real-World Example: HTTP Server Worker

```rust
use tokio::sync::mpsc;
use tokio::time::{sleep, Duration, interval};

struct Request {
    id: u32,
    data: String,
}

#[tokio::main]
async fn main() {
    let (shutdown_tx, mut shutdown_rx) = mpsc::channel::<()>(1);
    let (request_tx, mut request_rx) = mpsc::channel::<Request>(100);
    
    // Simulate request generator
    let request_gen = tokio::spawn(async move {
        for i in 1..=10 {
            request_tx.send(Request {
                id: i,
                data: format!("Request {}", i),
            }).await.unwrap();
            sleep(Duration::from_millis(80)).await;
        }
    });
    
    // Worker that processes requests
    let worker = tokio::spawn(async move {
        let mut tick = interval(Duration::from_millis(50));
        let mut processed = 0;
        
        loop {
            tokio::select! {
                // Health check / metrics tick
                _ = tick.tick() => {
                    println!("Health: {} requests processed", processed);
                }
                
                // Process incoming requests
                Some(request) = request_rx.recv() => {
                    println!("Processing request {}: {}", request.id, request.data);
                    sleep(Duration::from_millis(30)).await;
                    processed += 1;
                }
                
                // Shutdown signal
                _ = shutdown_rx.recv() => {
                    println!("Shutdown initiated, draining remaining requests...");
                    
                    // Drain remaining requests
                    while let Ok(request) = request_rx.try_recv() {
                        println!("Draining request {}", request.id);
                        processed += 1;
                    }
                    
                    break;
                }
            }
        }
        
        println!("Worker shutdown complete. Total processed: {}", processed);
    });
    
    // Wait for some requests to be processed
    sleep(Duration::from_millis(400)).await;
    
    // Initiate shutdown
    println!("\n=== Initiating shutdown ===");
    shutdown_tx.send(()).await.unwrap();
    
    // Wait for worker to complete
    worker.await.unwrap();
    request_gen.await.unwrap();
    
    println!("Application shutdown complete");
}
```

## Summary

The graceful shutdown pattern with `tokio::select!` provides:

1. **Concurrent listening**: Work and shutdown monitoring happen simultaneously
2. **Immediate response**: Shutdown signals are detected without delay
3. **Clean termination**: Cleanup code runs before task exits
4. **Resource management**: Proper closure of connections, files, etc.
5. **Predictable behavior**: Clear shutdown flow is easy to reason about

This pattern is fundamental to building robust async applications in Rust and should be used whenever you have long-running tasks that need controlled shutdown.

### Key Takeaway

Always structure worker tasks with `select!` to race between:
- **Work operations** (periodic tasks, incoming requests, etc.)
- **Shutdown signals** (via channels)

This ensures your application can shut down cleanly at any time while maintaining data integrity and proper resource cleanup.