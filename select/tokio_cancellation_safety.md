# Handling Cancellation-Unsafe Operations in `tokio::select!`

## Overview

This code demonstrates how to correctly handle **cancellation-unsafe operations** when using `tokio::select!`. Understanding cancellation safety is critical for writing robust async Rust code that doesn't lose data or leave operations incomplete.

## The Cancellation Safety Problem

When you use `.await` inside a `select!` branch, if another branch becomes ready while waiting, the current future is **dropped/cancelled**. This creates a serious problem:

### Unsafe Pattern ❌

```rust
tokio::select! {
    Some(data) = rx.recv() => {
        // PROBLEM: If shutdown signal arrives here,
        // process_data() is cancelled mid-execution!
        process_data(data).await;
    }
    _ = shutdown_rx.recv() => {
        println!("Shutting down");
    }
}
```

**What goes wrong:**
1. Message received from channel (removed from channel)
2. Processing starts: `process_data(data).await`
3. Shutdown signal arrives
4. `process_data()` future is **dropped** - processing incomplete!
5. Data is **lost** - already removed from channel, never fully processed

## The Solution: Two-Phase Approach

The key insight is to **separate receiving from processing**:

1. **Phase 1** (cancellable): Use `select!` to receive data OR shutdown signal
2. **Phase 2** (non-cancellable): Process data outside `select!` where it can't be cancelled

### Safe Pattern ✅

```rust
loop {
    // Phase 1: Receive (cancellable)
    let data = tokio::select! {
        data = rx.recv() => data,
        _ = shutdown_rx.recv() => {
            shutdown = true;
            None
        }
    };
    
    // Phase 2: Process (non-cancellable)
    if let Some(data) = data {
        process_data(data).await;  // Cannot be cancelled!
    }
    
    if shutdown || data.is_none() {
        break;
    }
}
```

## How It Works

### Step 1: Channel Setup

```rust
let (tx, mut rx) = mpsc::channel(32);
let (shutdown_tx, mut shutdown_rx) = mpsc::channel::<()>(1);
```

Two channels:
- **Data channel** (`tx`/`rx`): For normal work items
- **Shutdown channel** (`shutdown_tx`/`shutdown_rx`): For shutdown signals

### Step 2: Spawning a Producer

```rust
tokio::spawn(async move {
    tx.send("Data 1".to_string()).await.unwrap();
    sleep(Duration::from_millis(200)).await;
    shutdown_tx.send(()).await.unwrap();
});
```

The spawned task:
1. Sends a data message
2. Waits 200ms
3. Sends shutdown signal

### Step 3: The Cancellation-Safe Loop

```rust
let mut shutdown = false;

loop {
    // PHASE 1: Receive - This part IS cancellable
    let data = tokio::select! {
        data = rx.recv() => data,
        _ = shutdown_rx.recv() => {
            println!("Shutdown signal received");
            shutdown = true;
            None  // Return None to indicate no data to process
        }
    };
    
    // PHASE 2: Process - This part is NOT cancellable
    if let Some(data) = data {
        process_data(data).await;
    }
    
    // Exit after completing any in-progress work
    if shutdown || data.is_none() {
        println!("Shutting down");
        break;
    }
}
```

**Key mechanics:**

1. **`select!` only receives**: The `select!` block returns immediately with either data or a shutdown signal
2. **Processing happens outside**: Once data is received, it's stored in the `data` variable
3. **No cancellation possible**: The `process_data(data).await` is outside `select!`, so no other branch can interrupt it
4. **Graceful completion**: Even if shutdown is signaled, current data processing finishes first

## Execution Flow Example

Let's trace through what happens:

### Timeline

```
Time    Event
----    -----
0ms     Loop starts, select! waits
0ms     "Data 1" received from rx
0ms     data = Some("Data 1"), exits select!
0ms     Starts processing "Data 1"
50ms    "Data 1" processing completes
50ms    Loop restarts, select! waits again
200ms   Shutdown signal received
200ms   shutdown = true, data = None
200ms   No data to process, shutdown = true
200ms   Breaks loop, prints "Shutting down"
```

### What if Shutdown Arrives During Processing?

```
Time    Event
----    -----
0ms     "Data 1" received
0ms     Starts processing "Data 1"
25ms    ⚠️ Shutdown signal arrives (but processing continues!)
50ms    "Data 1" processing completes
50ms    Now checks shutdown flag, breaks loop
```

**Critical point**: The shutdown signal is received, but processing is **not interrupted** because it's outside the `select!` block.

## Complete Code

```rust
use tokio::sync::mpsc;
use tokio::time::{sleep, Duration};

async fn process_data(data: String) {
    println!("Processing: {}", data);
    sleep(Duration::from_millis(50)).await;
    println!("Done processing: {}", data);
}

#[tokio::main]
async fn main() {
    let (tx, mut rx) = mpsc::channel(32);
    let (shutdown_tx, mut shutdown_rx) = mpsc::channel::<()>(1);
    
    tokio::spawn(async move {
        tx.send("Data 1".to_string()).await.unwrap();
        sleep(Duration::from_millis(200)).await;
        shutdown_tx.send(()).await.unwrap();
    });
    
    let mut shutdown = false;
    
    loop {
        // PHASE 1: Receive data OR shutdown signal (cancellable)
        let data = tokio::select! {
            data = rx.recv() => data,
            _ = shutdown_rx.recv() => {
                println!("Shutdown signal received");
                shutdown = true;
                None
            }
        };
        
        // PHASE 2: Process data (NOT cancellable)
        if let Some(data) = data {
            process_data(data).await;
        }
        
        // Exit after processing current data
        if shutdown || data.is_none() {
            println!("Shutting down");
            break;
        }
    }
}
```

## Expected Output

```
Processing: Data 1
Done processing: Data 1
Shutdown signal received
Shutting down
```

Notice that "Done processing" appears even though the shutdown signal arrives after only 200ms - the processing (which takes 50ms starting from 0ms) completes successfully.

## Comparison: Unsafe vs Safe

### Unsafe Version (Processing Inside `select!`)

```rust
loop {
    tokio::select! {
        data = rx.recv() => {
            if let Some(data) = data {
                // ❌ Can be cancelled if shutdown arrives!
                process_data(data).await;
            }
        }
        _ = shutdown_rx.recv() => {
            println!("Shutting down");
            break;
        }
    }
}
```

**Problems:**
- Processing can be interrupted mid-way
- Data already removed from channel is lost
- Partial work may leave system in inconsistent state

**Possible output:**
```
Processing: Data 1
Shutting down
```
(Notice "Done processing" is missing - work was cancelled!)

### Safe Version (Processing Outside `select!`)

```rust
loop {
    let data = tokio::select! {
        data = rx.recv() => data,
        _ = shutdown_rx.recv() => {
            shutdown = true;
            None
        }
    };
    
    // ✅ Cannot be cancelled - outside select!
    if let Some(data) = data {
        process_data(data).await;
    }
    
    if shutdown || data.is_none() {
        break;
    }
}
```

**Benefits:**
- Processing always completes once started
- No data loss
- Graceful shutdown with work completion
- System consistency maintained

**Output:**
```
Processing: Data 1
Done processing: Data 1
Shutdown signal received
Shutting down
```

## What is Cancellation Safety?

A future is **cancellation-safe** if dropping it (stopping it mid-execution) doesn't cause problems:

### Cancellation-Safe Operations ✅

- **`mpsc::Receiver::recv()`**: Can be cancelled safely - message stays in channel
- **`oneshot::Receiver::recv()`**: Can be cancelled - value remains available
- **`sleep()`**: Can be cancelled - just stops waiting
- **Reading from a file** (with proper error handling): Can retry from last position

### Cancellation-Unsafe Operations ❌

- **Processing after receiving**: Data already removed, can't be recovered if cancelled
- **Database transactions**: Partial writes may occur
- **Multi-step workflows**: May leave system in inconsistent state
- **Network requests with side effects**: Request may complete server-side but client doesn't see result
- **File writes**: Partial data may be written

## Real-World Example: Database Operations

Here's a practical example with database operations:

```rust
use tokio::sync::mpsc;

async fn save_to_database(data: String) {
    println!("Starting database transaction for: {}", data);
    // Simulate multi-step database operation
    tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
    println!("Database transaction committed for: {}", data);
}

#[tokio::main]
async fn main() {
    let (tx, mut rx) = mpsc::channel(32);
    let (shutdown_tx, mut shutdown_rx) = mpsc::channel::<()>(1);
    
    // Simulate multiple data items
    tokio::spawn(async move {
        for i in 1..=5 {
            tx.send(format!("Record {}", i)).await.unwrap();
            tokio::time::sleep(tokio::time::Duration::from_millis(50)).await;
        }
        tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
        shutdown_tx.send(()).await.unwrap();
    });
    
    let mut shutdown = false;
    
    loop {
        // Receive next item OR shutdown signal
        let data = tokio::select! {
            data = rx.recv() => data,
            _ = shutdown_rx.recv() => {
                println!("Shutdown requested - finishing current transaction...");
                shutdown = true;
                None
            }
        };
        
        // Process completely (database transaction cannot be interrupted)
        if let Some(data) = data {
            save_to_database(data).await;
        }
        
        if shutdown || data.is_none() {
            println!("All transactions completed, shutting down");
            break;
        }
    }
}
```

This ensures:
- Every received record is fully saved to the database
- Transactions are never interrupted mid-way
- Graceful shutdown waits for current transaction to complete

## Best Practices

### 1. Identify Cancellation-Unsafe Operations

Ask yourself: "If this `.await` is cancelled, will data be lost or the system be inconsistent?"

If yes, move it outside `select!`.

### 2. Use the Two-Phase Pattern

```rust
// Phase 1: Receive (cancellable)
let item = tokio::select! { ... };

// Phase 2: Process (not cancellable)
if let Some(item) = item {
    process(item).await;
}
```

### 3. Document Cancellation Safety

```rust
// SAFETY: process_data() must complete once data is received
// from the channel, so it's called outside the select! block
if let Some(data) = data {
    process_data(data).await;
}
```

### 4. Consider `biased` for Shutdown Priorities

```rust
tokio::select! {
    biased;
    
    // Check shutdown first
    _ = shutdown_rx.recv() => { ... }
    
    // Then check for work
    data = rx.recv() => { ... }
}
```

This checks for shutdown before accepting new work.

### 5. Use Explicit Shutdown Flags

```rust
let mut shutdown = false;

// Instead of breaking immediately, set flag
if shutdown_condition {
    shutdown = true;
}

// Break after completing current work
if shutdown {
    break;
}
```

## When You Don't Need This Pattern

You can process inside `select!` when:

1. **Operations are idempotent**: Can be safely retried if cancelled
2. **No state changes**: Pure computation with no side effects
3. **Immediate shutdown required**: Cancelling work is the desired behavior
4. **Operations are naturally cancellation-safe**: Like reading from a file descriptor

## Summary

| Aspect | Unsafe Pattern | Safe Pattern |
|--------|---------------|--------------|
| **Processing location** | Inside `select!` | Outside `select!` |
| **Can be cancelled** | Yes ❌ | No ✅ |
| **Data loss risk** | High | None |
| **Code complexity** | Simple | Slightly more complex |
| **Use case** | Immediate shutdown | Graceful shutdown |

The key principle: **Separate what can be cancelled (receiving) from what must not be cancelled (processing)**.

By following this pattern, you ensure that once data is received from a channel, it will be fully processed, even if a shutdown signal or other event occurs. This is essential for building reliable async applications that don't lose data or leave operations incomplete.