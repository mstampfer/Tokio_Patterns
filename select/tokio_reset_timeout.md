# Resetting Timeout Pattern with `tokio::select!`

## Overview

This code demonstrates a **resetting timeout pattern** - a technique where a timeout is continuously reset each time activity occurs. This pattern is commonly used for detecting inactivity, implementing keep-alive mechanisms, or handling idle connections.

## The Resetting Timeout Pattern

Unlike a simple fixed timeout, a resetting timeout:

1. **Starts a countdown** when initialized
2. **Resets the countdown** every time activity occurs (message received)
3. **Triggers only on inactivity** - fires when no activity happens within the timeout window
4. **Adapts to activity patterns** - active streams never timeout, idle ones do

## Real-World Use Cases

- **Network connections**: Close idle connections after period of inactivity
- **Session management**: Expire user sessions after inactivity
- **Health checks**: Detect when services stop responding
- **Cache expiration**: Remove unused cache entries
- **Websocket keep-alive**: Detect disconnected clients
- **Database connection pools**: Close idle database connections

## How This Code Works

### Architecture Overview

```
Timeline:

0ms     ├─ Start, set deadline to 200ms
        │
80ms    ├─ Message 0 arrives → Reset deadline to 280ms
        │
160ms   ├─ Message 1 arrives → Reset deadline to 360ms
        │
240ms   ├─ Message 2 arrives → Reset deadline to 440ms
        │
        ├─ Long pause (300ms) begins...
        │
440ms   ├─ Deadline reached, no message!
        └─ TIMEOUT - Exit loop
```

## Code Breakdown

### Step 1: Setup Channel and Spawn Producer

```rust
let (tx, mut rx) = mpsc::channel::<String>(32);

tokio::spawn(async move {
    for i in 0..3 {
        tokio::time::sleep(Duration::from_millis(80)).await;
        tx.send(format!("Message {}", i)).await.unwrap();
    }
    // Then pause for longer than timeout
    tokio::time::sleep(Duration::from_millis(300)).await;
    tx.send("Final message".to_string()).await.unwrap();
});
```

**Producer behavior:**
1. Sends 3 messages, one every 80ms (at 80ms, 160ms, 240ms)
2. Pauses for 300ms (exceeds the 200ms timeout)
3. Sends a final message (which won't be received due to timeout)

### Step 2: Initialize Timeout Parameters

```rust
let timeout_duration = Duration::from_millis(200);
let mut deadline = Instant::now() + timeout_duration;
```

**Key variables:**
- `timeout_duration`: Fixed timeout window (200ms of inactivity)
- `deadline`: Mutable variable tracking when timeout should fire
- Initial deadline set to "now + 200ms"

### Step 3: The Select Loop with Resetting Timeout

```rust
loop {
    tokio::select! {
        msg = rx.recv() => {
            if let Some(msg) = msg {
                println!("Received: {}", msg);
                // CRITICAL: Reset timeout
                deadline = Instant::now() + timeout_duration;
            } else {
                // Channel closed
                break;
            }
        }
        _ = sleep_until(deadline) => {
            println!("Timeout - no messages for {:?}", timeout_duration);
            break;
        }
    }
}
```

**How it works:**

1. **Two racing branches**:
   - Message reception: `rx.recv()`
   - Timeout: `sleep_until(deadline)`

2. **Message branch executes when**:
   - A message arrives from the channel
   - Prints the message
   - **Resets the deadline**: `deadline = Instant::now() + timeout_duration`
   - Loop continues with new deadline

3. **Timeout branch executes when**:
   - No message received before `deadline` is reached
   - Prints timeout message
   - Breaks the loop (ends program)

## Why `sleep_until()` Instead of `sleep()`?

### Using `sleep()` (Problematic)

```rust
// ❌ BAD: Can panic if deadline has passed
_ = sleep(deadline - Instant::now()) => {
    // Subtraction can underflow!
}
```

**Problems:**
- If `Instant::now()` > `deadline`, subtraction panics
- Requires manual checking for past deadlines
- More complex error handling

### Using `sleep_until()` (Correct)

```rust
// ✅ GOOD: Handles past deadlines gracefully
_ = sleep_until(deadline) => {
    // If deadline has passed, completes immediately
}
```

**Benefits:**
- Never panics
- If deadline is in the past, completes immediately
- Cleaner, more idiomatic code

## Complete Code

```rust
use tokio::sync::mpsc;
use tokio::time::{sleep_until, Duration, Instant};

#[tokio::main]
async fn main() {
    let (tx, mut rx) = mpsc::channel::<String>(32);
    
    tokio::spawn(async move {
        for i in 0..3 {
            tokio::time::sleep(Duration::from_millis(80)).await;
            tx.send(format!("Message {}", i)).await.unwrap();
        }
        // Then pause for longer than timeout
        tokio::time::sleep(Duration::from_millis(300)).await;
        tx.send("Final message".to_string()).await.unwrap();
    });
    
    let timeout_duration = Duration::from_millis(200);
    let mut deadline = Instant::now() + timeout_duration;
    
    loop {
        tokio::select! {
            msg = rx.recv() => {
                if let Some(msg) = msg {
                    println!("Received: {}", msg);
                    // Reset timeout
                    deadline = Instant::now() + timeout_duration;
                } else {
                    // Channel closed
                    break;
                }
            }
            _ = sleep_until(deadline) => {
                println!("Timeout - no messages for {:?}", timeout_duration);
                break;
            }
        }
    }
}
```

## Cargo.toml Setup

```toml
[package]
name = "timeout-example"
version = "0.1.0"
edition = "2021"

[dependencies]
tokio = { version = "1", features = ["full"] }
```

Or with minimal features:

```toml
[dependencies]
tokio = { version = "1", features = ["rt-multi-thread", "macros", "time", "sync"] }
```

## Expected Output

```
Received: Message 0
Received: Message 1
Received: Message 2
Timeout - no messages for 200ms
```

**What happened:**
1. Message 0 received at ~80ms → deadline reset to ~280ms
2. Message 1 received at ~160ms → deadline reset to ~360ms
3. Message 2 received at ~240ms → deadline reset to ~440ms
4. No message for 200ms (from 240ms to 440ms)
5. Timeout fires at ~440ms
6. "Final message" never received (sent after timeout)

## Detailed Execution Timeline

```
Time    Event                           Deadline
----    -----                           --------
0ms     Loop starts                     200ms
        select! waits...

80ms    Message 0 arrives
        "Received: Message 0"
        Deadline reset                  280ms
        select! waits...

160ms   Message 1 arrives
        "Received: Message 1"
        Deadline reset                  360ms
        select! waits...

240ms   Message 2 arrives
        "Received: Message 2"
        Deadline reset                  440ms
        select! waits...
        
        [300ms pause begins]
        [No messages...]

440ms   Deadline reached!
        "Timeout - no messages for 200ms"
        break; → Exit loop

540ms   Final message sent
        (but receiver already exited)
```

## Alternative Implementation: Using `timeout()`

Another approach using Tokio's `timeout()` function:

```rust
use tokio::sync::mpsc;
use tokio::time::{timeout, Duration};

#[tokio::main]
async fn main() {
    let (tx, mut rx) = mpsc::channel::<String>(32);
    
    tokio::spawn(async move {
        for i in 0..3 {
            tokio::time::sleep(Duration::from_millis(80)).await;
            tx.send(format!("Message {}", i)).await.unwrap();
        }
        tokio::time::sleep(Duration::from_millis(300)).await;
        tx.send("Final message".to_string()).await.unwrap();
    });
    
    let timeout_duration = Duration::from_millis(200);
    
    loop {
        match timeout(timeout_duration, rx.recv()).await {
            Ok(Some(msg)) => {
                println!("Received: {}", msg);
                // Timeout automatically resets on next iteration
            }
            Ok(None) => {
                println!("Channel closed");
                break;
            }
            Err(_) => {
                println!("Timeout - no messages for {:?}", timeout_duration);
                break;
            }
        }
    }
}
```

**Pros:**
- Simpler - no manual deadline management
- Automatic reset on each iteration
- More concise code

**Cons:**
- Less control over timeout behavior
- Cannot easily implement variable timeout durations
- Timeout resets on every loop iteration, not just on message

## Pattern Variations

### 1. Different Timeout After First Message

```rust
let initial_timeout = Duration::from_secs(30);
let activity_timeout = Duration::from_secs(5);
let mut first_message = true;

let mut deadline = Instant::now() + initial_timeout;

loop {
    tokio::select! {
        msg = rx.recv() => {
            if let Some(msg) = msg {
                println!("Received: {}", msg);
                
                // Use shorter timeout after first message
                let timeout_to_use = if first_message {
                    first_message = false;
                    activity_timeout
                } else {
                    activity_timeout
                };
                
                deadline = Instant::now() + timeout_to_use;
            } else {
                break;
            }
        }
        _ = sleep_until(deadline) => {
            println!("Inactivity timeout");
            break;
        }
    }
}
```

### 2. Timeout with Warning Before Disconnect

```rust
let warning_time = Duration::from_secs(25);
let disconnect_time = Duration::from_secs(30);
let mut deadline = Instant::now() + disconnect_time;
let mut warning_sent = false;

loop {
    let time_until_deadline = deadline.saturating_duration_since(Instant::now());
    
    if !warning_sent && time_until_deadline <= warning_time {
        println!("Warning: Connection will timeout in {:?}", time_until_deadline);
        warning_sent = true;
    }
    
    tokio::select! {
        msg = rx.recv() => {
            if let Some(msg) = msg {
                println!("Received: {}", msg);
                deadline = Instant::now() + disconnect_time;
                warning_sent = false;  // Reset warning
            } else {
                break;
            }
        }
        _ = sleep_until(deadline) => {
            println!("Connection timeout - disconnecting");
            break;
        }
    }
}
```

### 3. Exponential Backoff Timeout

```rust
let mut timeout_duration = Duration::from_millis(200);
let max_timeout = Duration::from_secs(10);
let mut deadline = Instant::now() + timeout_duration;

loop {
    tokio::select! {
        msg = rx.recv() => {
            if let Some(msg) = msg {
                println!("Received: {}", msg);
                // Reset to initial timeout on activity
                timeout_duration = Duration::from_millis(200);
                deadline = Instant::now() + timeout_duration;
            } else {
                break;
            }
        }
        _ = sleep_until(deadline) => {
            println!("Timeout after {:?}", timeout_duration);
            
            // Increase timeout (exponential backoff)
            timeout_duration = (timeout_duration * 2).min(max_timeout);
            deadline = Instant::now() + timeout_duration;
            
            if timeout_duration >= max_timeout {
                println!("Max timeout reached, disconnecting");
                break;
            }
        }
    }
}
```

## Real-World Example: HTTP Keep-Alive

```rust
use tokio::sync::mpsc;
use tokio::time::{sleep_until, Duration, Instant};

struct HttpRequest {
    method: String,
    path: String,
}

#[tokio::main]
async fn main() {
    let (request_tx, mut request_rx) = mpsc::channel::<HttpRequest>(32);
    
    // Simulate client sending requests
    tokio::spawn(async move {
        // Active period
        for i in 0..3 {
            tokio::time::sleep(Duration::from_millis(500)).await;
            request_tx.send(HttpRequest {
                method: "GET".to_string(),
                path: format!("/api/data/{}", i),
            }).await.unwrap();
        }
        
        // Client becomes idle (no requests for 5+ seconds)
        tokio::time::sleep(Duration::from_secs(6)).await;
        
        // Try to send another request (but connection will be closed)
        let _ = request_tx.send(HttpRequest {
            method: "GET".to_string(),
            path: "/api/final".to_string(),
        }).await;
    });
    
    // Server-side connection handler
    let keep_alive_timeout = Duration::from_secs(5);
    let mut deadline = Instant::now() + keep_alive_timeout;
    let mut request_count = 0;
    
    println!("HTTP connection established");
    
    loop {
        tokio::select! {
            request = request_rx.recv() => {
                match request {
                    Some(req) => {
                        request_count += 1;
                        println!("[Request #{}] {} {}", request_count, req.method, req.path);
                        
                        // Simulate processing
                        tokio::time::sleep(Duration::from_millis(100)).await;
                        println!("[Request #{}] Response sent", request_count);
                        
                        // Reset keep-alive timeout
                        deadline = Instant::now() + keep_alive_timeout;
                    }
                    None => {
                        println!("Client closed connection");
                        break;
                    }
                }
            }
            _ = sleep_until(deadline) => {
                println!("Keep-alive timeout - closing idle connection");
                println!("Total requests handled: {}", request_count);
                break;
            }
        }
    }
    
    println!("Connection closed");
}
```

### Output:
```
HTTP connection established
[Request #1] GET /api/data/0
[Request #1] Response sent
[Request #2] GET /api/data/1
[Request #2] Response sent
[Request #3] GET /api/data/2
[Request #3] Response sent
Keep-alive timeout - closing idle connection
Total requests handled: 3
Connection closed
```

## Best Practices

### 1. Choose Appropriate Timeout Durations

```rust
// Too short: May timeout during normal activity
let timeout = Duration::from_millis(10);  // ❌ Too aggressive

// Too long: Resources held unnecessarily
let timeout = Duration::from_secs(3600);  // ❌ Too lenient

// Just right: Based on expected activity patterns
let timeout = Duration::from_secs(30);    // ✅ Reasonable
```

### 2. Always Reset on Activity

```rust
// ✅ Good: Reset on every message
msg = rx.recv() => {
    if let Some(msg) = msg {
        process(msg);
        deadline = Instant::now() + timeout_duration;  // Reset!
    }
}

// ❌ Bad: Forgot to reset
msg = rx.recv() => {
    if let Some(msg) = msg {
        process(msg);
        // Missing reset - will timeout incorrectly!
    }
}
```

### 3. Handle Channel Closure

```rust
msg = rx.recv() => {
    match msg {
        Some(msg) => {
            // Process and reset
        }
        None => {
            // Channel closed - clean exit
            println!("Channel closed gracefully");
            break;
        }
    }
}
```

### 4. Log Timeout Events

```rust
_ = sleep_until(deadline) => {
    eprintln!("Inactivity timeout after {:?} - last activity: {:?} ago",
              timeout_duration,
              Instant::now().duration_since(deadline - timeout_duration));
    break;
}
```

## Common Pitfalls

### Pitfall 1: Not Using `mut` for Deadline

```rust
// ❌ Error: Cannot assign to immutable variable
let deadline = Instant::now() + timeout_duration;
deadline = Instant::now() + timeout_duration;  // Compile error!

// ✅ Correct: Use mut
let mut deadline = Instant::now() + timeout_duration;
deadline = Instant::now() + timeout_duration;  // Works!
```

### Pitfall 2: Using Fixed Duration Instead of Deadline

```rust
// ❌ Bad: Timeout doesn't actually reset
let timeout_duration = Duration::from_secs(5);
loop {
    tokio::select! {
        msg = rx.recv() => { /* ... */ }
        _ = tokio::time::sleep(timeout_duration) => {
            // This always waits the full duration!
            // Doesn't implement resetting timeout!
        }
    }
}
```

### Pitfall 3: Resetting to Fixed Time Instead of Current Time

```rust
// ❌ Bad: Using stale time reference
let start = Instant::now();
deadline = start + timeout_duration;  // Always relative to start!

// ✅ Good: Using current time
deadline = Instant::now() + timeout_duration;  // Relative to now!
```

## Summary

The resetting timeout pattern with `tokio::select!` provides:

1. **Inactivity detection**: Triggers only when no activity occurs
2. **Dynamic adaptation**: Active streams never timeout
3. **Resource management**: Frees resources from idle connections
4. **Flexible timing**: Easy to adjust timeout durations
5. **Clean implementation**: Simple, readable code with `select!`

### Key Components

- **`sleep_until(deadline)`**: Waits until a specific instant
- **Mutable deadline**: Updated on each activity
- **`Instant::now() + Duration`**: Creates new deadlines
- **`select!` racing**: Concurrent wait for activity or timeout

This pattern is essential for building robust networked applications, implementing keep-alive mechanisms, and managing long-lived resources efficiently.