# Selecting from Different Channel Types in Tokio

## Overview

This code demonstrates how to use `tokio::select!` to concurrently wait on three different types of Tokio channels: **MPSC**, **Oneshot**, and **Broadcast**. Each channel type has different characteristics and use cases, and they have slightly different APIs for receiving messages.

## The Three Channel Types

### 1. MPSC (Multi-Producer, Single-Consumer)

```rust
let (mpsc_tx, mut mpsc_rx) = mpsc::channel::<String>(32);
```

**Characteristics:**
- Multiple senders can send to one receiver
- Buffered: Can hold up to `capacity` messages
- Returns `Option<T>`: `Some(value)` when message received, `None` when all senders dropped
- **Receive method**: `rx.recv().await`

**Use cases:**
- Task queues
- Fan-in pattern (multiple producers, single consumer)
- General-purpose async communication

### 2. Oneshot (Single-Producer, Single-Consumer)

```rust
let (oneshot_tx, oneshot_rx) = oneshot::channel::<i32>();
```

**Characteristics:**
- One sender, one receiver
- Can send exactly **one** message
- Receiver consumes itself when receiving
- Returns `Result<T, RecvError>`: `Ok(value)` when message received, `Err` if sender dropped
- **Receive method**: `rx.await` (no `.recv()` call - receiver IS the future!)

**Use cases:**
- Request/response patterns
- Returning results from spawned tasks
- One-time notifications
- Future cancellation tokens

### 3. Broadcast (Multi-Producer, Multi-Consumer)

```rust
let (broadcast_tx, mut broadcast_rx) = broadcast::channel::<bool>(16);
```

**Characteristics:**
- Multiple senders, multiple receivers
- Each receiver gets a **copy** of every message
- Receivers can lag behind (old messages dropped if capacity exceeded)
- Returns `Result<T, RecvError>`: `Ok(value)` when message received, `Err` for various error conditions
- **Receive method**: `rx.recv().await`

**Use cases:**
- Event notifications to multiple listeners
- Pub/sub patterns
- Broadcasting configuration changes
- Shutdown signals to multiple tasks

## How the Code Works

### Step 1: Create All Three Channel Types

```rust
let (mpsc_tx, mut mpsc_rx) = mpsc::channel::<String>(32);
let (oneshot_tx, oneshot_rx) = oneshot::channel::<i32>();
let (broadcast_tx, mut broadcast_rx) = broadcast::channel::<bool>(16);
```

Three independent channels are created, each with its own semantics.

### Step 2: Spawn a Task to Send on Oneshot

```rust
tokio::spawn(async move {
    sleep(Duration::from_millis(100)).await;
    oneshot_tx.send(42).unwrap();
});
```

A task waits 100ms then sends a value through the oneshot channel. The other channels (MPSC and Broadcast) remain empty in this example.

### Step 3: Select from All Three Channels

```rust
tokio::select! {
    msg = mpsc_rx.recv() => {
        if let Some(msg) = msg {
            println!("MPSC: {}", msg);
        }
    }
    result = oneshot_rx => {
        match result {
            Ok(value) => println!("Oneshot: {}", value),
            Err(_) => println!("Oneshot sender dropped"),
        }
    }
    result = broadcast_rx.recv() => {
        match result {
            Ok(flag) => println!("Broadcast: {}", flag),
            Err(_) => println!("Broadcast error"),
        }
    }
}
```

**What happens:**

1. **`select!` races all three channels**: Waits for whichever becomes ready first
2. **MPSC branch**: `mpsc_rx.recv()` returns `Option<String>`
3. **Oneshot branch**: `oneshot_rx` (the receiver itself) returns `Result<i32, RecvError>`
4. **Broadcast branch**: `broadcast_rx.recv()` returns `Result<bool, RecvError>`
5. **First ready wins**: The oneshot channel becomes ready after 100ms
6. **Prints**: `"Oneshot: 42"`

## Key API Differences

Notice the subtle but important differences in how each channel type is used:

### MPSC - Call `.recv()`
```rust
msg = mpsc_rx.recv() => {
    if let Some(msg) = msg {
        println!("MPSC: {}", msg);
    }
}
```
- **Pattern**: `rx.recv()`
- **Returns**: `Option<T>`
- **None means**: All senders dropped, channel closed

### Oneshot - No `.recv()`, await receiver directly
```rust
result = oneshot_rx => {
    match result {
        Ok(value) => println!("Oneshot: {}", value),
        Err(_) => println!("Oneshot sender dropped"),
    }
}
```
- **Pattern**: `rx` (not `rx.recv()`)
- **Returns**: `Result<T, RecvError>`
- **Why different**: Oneshot receiver implements `Future` directly
- **Consumes receiver**: Can only receive once

### Broadcast - Call `.recv()`
```rust
result = broadcast_rx.recv() => {
    match result {
        Ok(flag) => println!("Broadcast: {}", flag),
        Err(_) => println!("Broadcast error"),
    }
}
```
- **Pattern**: `rx.recv()`
- **Returns**: `Result<T, RecvError>`
- **Errors include**: `RecvError::Lagged` (receiver fell behind), `RecvError::Closed`

## Complete Code

```rust
use tokio::sync::{mpsc, oneshot, broadcast};
use tokio::time::{sleep, Duration};

#[tokio::main]
async fn main() {
    let (mpsc_tx, mut mpsc_rx) = mpsc::channel::<String>(32);
    let (oneshot_tx, oneshot_rx) = oneshot::channel::<i32>();
    let (broadcast_tx, mut broadcast_rx) = broadcast::channel::<bool>(16);
    
    tokio::spawn(async move {
        sleep(Duration::from_millis(100)).await;
        oneshot_tx.send(42).unwrap();
    });
    
    tokio::select! {
        msg = mpsc_rx.recv() => {
            if let Some(msg) = msg {
                println!("MPSC: {}", msg);
            }
        }
        result = oneshot_rx => {
            match result {
                Ok(value) => println!("Oneshot: {}", value),
                Err(_) => println!("Oneshot sender dropped"),
            }
        }
        result = broadcast_rx.recv() => {
            match result {
                Ok(flag) => println!("Broadcast: {}", flag),
                Err(_) => println!("Broadcast error"),
            }
        }
    }
}
```

## Expected Output

```
Oneshot: 42
```

Since only the oneshot channel receives a message, its branch executes and prints the value.

## Comprehensive Example: All Channels Active

Here's a more complete example that demonstrates all three channel types with actual messages:

```rust
use tokio::sync::{mpsc, oneshot, broadcast};
use tokio::time::{sleep, Duration};

#[tokio::main]
async fn main() {
    let (mpsc_tx, mut mpsc_rx) = mpsc::channel::<String>(32);
    let (oneshot_tx, oneshot_rx) = oneshot::channel::<i32>();
    let (broadcast_tx, mut broadcast_rx) = broadcast::channel::<bool>(16);
    
    // Spawn tasks to send on different channels at different times
    tokio::spawn(async move {
        sleep(Duration::from_millis(150)).await;
        mpsc_tx.send("Hello from MPSC".to_string()).await.unwrap();
    });
    
    tokio::spawn(async move {
        sleep(Duration::from_millis(100)).await;
        oneshot_tx.send(42).unwrap();
    });
    
    tokio::spawn(async move {
        sleep(Duration::from_millis(200)).await;
        broadcast_tx.send(true).unwrap();
    });
    
    // First select: Oneshot will win (100ms)
    println!("First select:");
    tokio::select! {
        msg = mpsc_rx.recv() => {
            if let Some(msg) = msg {
                println!("  MPSC: {}", msg);
            }
        }
        result = oneshot_rx => {
            match result {
                Ok(value) => println!("  Oneshot: {}", value),
                Err(_) => println!("  Oneshot sender dropped"),
            }
        }
        result = broadcast_rx.recv() => {
            match result {
                Ok(flag) => println!("  Broadcast: {}", flag),
                Err(_) => println!("  Broadcast error"),
            }
        }
    }
    
    // Second select: MPSC will win (150ms total, oneshot already consumed)
    println!("\nSecond select:");
    tokio::select! {
        msg = mpsc_rx.recv() => {
            if let Some(msg) = msg {
                println!("  MPSC: {}", msg);
            }
        }
        result = broadcast_rx.recv() => {
            match result {
                Ok(flag) => println!("  Broadcast: {}", flag),
                Err(_) => println!("  Broadcast error"),
            }
        }
    }
    
    // Third select: Broadcast will win (200ms total)
    println!("\nThird select:");
    tokio::select! {
        msg = mpsc_rx.recv() => {
            if let Some(msg) = msg {
                println!("  MPSC: {}", msg);
            }
        }
        result = broadcast_rx.recv() => {
            match result {
                Ok(flag) => println!("  Broadcast: {}", flag),
                Err(_) => println!("  Broadcast error"),
            }
        }
    }
}
```

### Output:
```
First select:
  Oneshot: 42

Second select:
  MPSC: Hello from MPSC

Third select:
  Broadcast: true
```

## Channel Type Comparison Table

| Feature | MPSC | Oneshot | Broadcast |
|---------|------|---------|-----------|
| **Senders** | Multiple | Single | Multiple |
| **Receivers** | Single | Single | Multiple |
| **Messages** | Unlimited | One only | Unlimited |
| **Buffer** | Yes (capacity) | No | Yes (capacity) |
| **Receive API** | `rx.recv()` | `rx` (await directly) | `rx.recv()` |
| **Return type** | `Option<T>` | `Result<T, RecvError>` | `Result<T, RecvError>` |
| **Clone sender** | ✅ Yes | ❌ No | ✅ Yes |
| **Clone receiver** | ❌ No | ❌ No | ✅ Yes |
| **Consumes receiver** | ❌ No | ✅ Yes | ❌ No |

## Practical Use Case: Server with Multiple Event Sources

Here's a realistic example showing how you might use all three channel types together:

```rust
use tokio::sync::{mpsc, oneshot, broadcast};
use tokio::time::{sleep, Duration};

enum Request {
    Get(String),
    Put(String, String),
}

#[tokio::main]
async fn main() {
    // MPSC: For incoming client requests
    let (request_tx, mut request_rx) = mpsc::channel::<Request>(100);
    
    // Oneshot: For receiving initialization status
    let (init_tx, init_rx) = oneshot::channel::<bool>();
    
    // Broadcast: For shutdown signal to all tasks
    let (shutdown_tx, mut shutdown_rx) = broadcast::channel::<()>(1);
    
    // Simulate initialization
    tokio::spawn(async move {
        println!("Initializing server...");
        sleep(Duration::from_millis(50)).await;
        init_tx.send(true).unwrap();
        println!("Server initialized");
    });
    
    // Simulate client requests
    let request_sender = request_tx.clone();
    tokio::spawn(async move {
        sleep(Duration::from_millis(100)).await;
        request_sender.send(Request::Get("key1".to_string())).await.unwrap();
    });
    
    // Simulate shutdown signal
    tokio::spawn(async move {
        sleep(Duration::from_millis(200)).await;
        println!("Sending shutdown signal");
        shutdown_tx.send(()).unwrap();
    });
    
    // Main server loop
    loop {
        tokio::select! {
            // Handle client requests
            Some(request) = request_rx.recv() => {
                match request {
                    Request::Get(key) => println!("Processing GET for key: {}", key),
                    Request::Put(key, value) => println!("Processing PUT: {} = {}", key, value),
                }
            }
            
            // Wait for initialization (only happens once)
            result = init_rx => {
                match result {
                    Ok(success) if success => println!("Init complete, ready for requests"),
                    _ => println!("Init failed"),
                }
            }
            
            // Handle shutdown signal
            result = shutdown_rx.recv() => {
                match result {
                    Ok(_) => {
                        println!("Shutdown received, cleaning up...");
                        break;
                    }
                    Err(_) => println!("Shutdown channel error"),
                }
            }
        }
    }
    
    println!("Server stopped");
}
```

### Output:
```
Initializing server...
Server initialized
Init complete, ready for requests
Processing GET for key: key1
Sending shutdown signal
Shutdown received, cleaning up...
Server stopped
```

## Best Practices

### 1. Choose the Right Channel Type

- **Use MPSC** for general-purpose producer-consumer patterns
- **Use Oneshot** for request-response or one-time results
- **Use Broadcast** when multiple tasks need the same events

### 2. Handle All Result/Option Cases

```rust
// Good: Handle both Some and None
msg = rx.recv() => {
    match msg {
        Some(msg) => { /* process */ },
        None => { /* channel closed */ },
    }
}

// Good: Handle Ok and Err
result = oneshot_rx => {
    match result {
        Ok(value) => { /* use value */ },
        Err(_) => { /* sender dropped */ },
    }
}
```

### 3. Remember Oneshot Receivers Are Consumed

```rust
// This won't work - oneshot_rx consumed by first await
let value1 = oneshot_rx.await.unwrap();
let value2 = oneshot_rx.await.unwrap(); // ❌ Compile error!

// Oneshot can only be received once
tokio::select! {
    result = oneshot_rx => { /* This consumes oneshot_rx */ }
}
// Can't use oneshot_rx again here
```

### 4. Be Aware of Broadcast Lagging

```rust
// Broadcast receivers can lag if they don't keep up
result = broadcast_rx.recv() => {
    match result {
        Ok(value) => println!("Got: {:?}", value),
        Err(broadcast::error::RecvError::Lagged(n)) => {
            println!("Lagged by {} messages", n);
        }
        Err(_) => println!("Channel closed"),
    }
}
```

## Summary

When using `tokio::select!` with different channel types:

1. **MPSC**: Use `rx.recv()`, returns `Option<T>`, good for queues
2. **Oneshot**: Use `rx` directly (no `.recv()`), returns `Result<T, RecvError>`, good for one-time results
3. **Broadcast**: Use `rx.recv()`, returns `Result<T, RecvError>`, good for events to multiple listeners

The key insight is that while these channels have different semantics and use cases, they all work seamlessly with `tokio::select!`, allowing you to race between different types of async communication patterns in a single unified interface.