# Iterating Over Streams with `while let Some`

## Overview

This code demonstrates how to iterate over an **async stream** using the `while let Some` pattern. Streams are the async equivalent of iterators - they produce values over time asynchronously. The pattern shown here converts a Tokio channel receiver into a stream and processes each value as it arrives.

## What is a Stream?

A **Stream** is an asynchronous sequence of values, similar to how an **Iterator** is a synchronous sequence. The key differences:

| Feature | Iterator | Stream |
|---------|----------|--------|
| **Execution** | Synchronous | Asynchronous |
| **Next item** | `iterator.next()` | `stream.next().await` |
| **Trait** | `Iterator` | `Stream` |
| **Returns** | `Option<Item>` | `Future<Output = Option<Item>>` |
| **Use case** | In-memory collections | Async I/O, channels, events |

## Why Convert Channels to Streams?

Converting an `mpsc::Receiver` to a `Stream` enables:

1. **Stream combinators**: `map`, `filter`, `fold`, `collect`, etc.
2. **Unified interface**: Treat different async sources uniformly
3. **Composition**: Combine multiple streams easily
4. **Abstraction**: Work with the `Stream` trait instead of concrete types

## How This Code Works

### Step 1: Create Channel and Convert to Stream

```rust
let (tx, rx) = mpsc::channel(32);
let mut stream = ReceiverStream::new(rx);
```

**What happens:**
1. Create an MPSC channel with buffer size 32
2. `ReceiverStream::new(rx)` wraps the receiver, implementing the `Stream` trait
3. The stream can now use all stream methods from `StreamExt`

### Step 2: Spawn Producer Task

```rust
tokio::spawn(async move {
    for i in 0..5 {
        tx.send(i).await.unwrap();
    }
});
```

**Producer behavior:**
- Sends values 0, 1, 2, 3, 4 through the channel
- When the loop completes, `tx` is dropped
- Dropping `tx` closes the channel
- Closed channel causes stream to return `None`

### Step 3: Iterate Over Stream with `while let Some`

```rust
while let Some(value) = stream.next().await {
    println!("Got: {}", value);
}
```

**How it works:**

1. **`stream.next()`**: Returns a `Future<Output = Option<T>>`
2. **`.await`**: Waits for the future to resolve
3. **`Some(value)`**: Pattern matches when a value is available
4. **Loop body**: Processes the value
5. **`None`**: When stream ends (channel closed), pattern doesn't match, loop exits

This is the async equivalent of:
```rust
// Synchronous iterator version
while let Some(value) = iterator.next() {
    println!("Got: {}", value);
}
```

## Visual Flow Diagram

```
Producer Task                    Main Task (Stream Consumer)
    |                                    |
    ├─> send(0) ──────────────────────> stream.next().await
    |                                    └─> Some(0) → print "Got: 0"
    |                                    |
    ├─> send(1) ──────────────────────> stream.next().await
    |                                    └─> Some(1) → print "Got: 1"
    |                                    |
    ├─> send(2) ──────────────────────> stream.next().await
    |                                    └─> Some(2) → print "Got: 2"
    |                                    |
    ├─> send(3) ──────────────────────> stream.next().await
    |                                    └─> Some(3) → print "Got: 3"
    |                                    |
    ├─> send(4) ──────────────────────> stream.next().await
    |                                    └─> Some(4) → print "Got: 4"
    |                                    |
    └─> drops tx (closes channel) ────> stream.next().await
                                         └─> None → exit while loop
                                         |
                                         └─> Program ends
```

## Complete Code

```rust
use tokio::sync::mpsc;
use tokio_stream::wrappers::ReceiverStream;
use futures::StreamExt;

#[tokio::main]
async fn main() {
    let (tx, rx) = mpsc::channel(32);
    let mut stream = ReceiverStream::new(rx);
    
    tokio::spawn(async move {
        for i in 0..5 {
            tx.send(i).await.unwrap();
        }
    });
    
    while let Some(value) = stream.next().await {
        println!("Got: {}", value);
    }
}
```

## Cargo.toml Setup

```toml
[package]
name = "stream-example"
version = "0.1.0"
edition = "2021"

[dependencies]
tokio = { version = "1", features = ["full"] }
tokio-stream = "0.1"
futures = "0.3"
```

## Expected Output

```
Got: 0
Got: 1
Got: 2
Got: 3
Got: 4
```

Values are printed in order as they're received from the stream.

## Alternative Iteration Patterns

### 1. Using `for_each()`

```rust
use tokio::sync::mpsc;
use tokio_stream::wrappers::ReceiverStream;
use futures::StreamExt;

#[tokio::main]
async fn main() {
    let (tx, rx) = mpsc::channel(32);
    let stream = ReceiverStream::new(rx);
    
    tokio::spawn(async move {
        for i in 0..5 {
            tx.send(i).await.unwrap();
        }
    });
    
    stream.for_each(|value| async move {
        println!("Got: {}", value);
    }).await;
}
```

### 2. Using `collect()`

```rust
use tokio::sync::mpsc;
use tokio_stream::wrappers::ReceiverStream;
use futures::StreamExt;

#[tokio::main]
async fn main() {
    let (tx, rx) = mpsc::channel(32);
    let stream = ReceiverStream::new(rx);
    
    tokio::spawn(async move {
        for i in 0..5 {
            tx.send(i).await.unwrap();
        }
    });
    
    let values: Vec<i32> = stream.collect().await;
    println!("Collected: {:?}", values);
}
```

Output: `Collected: [0, 1, 2, 3, 4]`

### 3. Manual Loop with `match`

```rust
loop {
    match stream.next().await {
        Some(value) => {
            println!("Got: {}", value);
        }
        None => {
            println!("Stream ended");
            break;
        }
    }
}
```

## Stream Combinators Example

Once you have a stream, you can use powerful combinators:

```rust
use tokio::sync::mpsc;
use tokio_stream::wrappers::ReceiverStream;
use futures::StreamExt;

#[tokio::main]
async fn main() {
    let (tx, rx) = mpsc::channel(32);
    let stream = ReceiverStream::new(rx);
    
    tokio::spawn(async move {
        for i in 0..10 {
            tx.send(i).await.unwrap();
        }
    });
    
    // Chain multiple stream operations
    let mut processed_stream = stream
        .filter(|x| futures::future::ready(x % 2 == 0))  // Only even numbers
        .map(|x| x * 2)                                    // Double them
        .take(3);                                          // Take first 3
    
    while let Some(value) = processed_stream.next().await {
        println!("Processed: {}", value);
    }
}
```

Output:
```
Processed: 0
Processed: 4
Processed: 8
```

## Comparison: Channel Receiver vs Stream

### Using Receiver Directly

```rust
let (tx, mut rx) = mpsc::channel(32);

// Spawn producer...

while let Some(value) = rx.recv().await {
    println!("Got: {}", value);
}
```

**Characteristics:**
- Direct channel API
- Simple and straightforward
- No extra dependencies
- No combinator support

### Using ReceiverStream

```rust
let (tx, rx) = mpsc::channel(32);
let mut stream = ReceiverStream::new(rx);

// Spawn producer...

while let Some(value) = stream.next().await {
    println!("Got: {}", value);
}
```

**Characteristics:**
- Stream trait implementation
- Access to stream combinators
- Can be composed with other streams
- Unified interface

## Practical Example: Processing Events

```rust
use tokio::sync::mpsc;
use tokio_stream::wrappers::ReceiverStream;
use futures::StreamExt;
use tokio::time::{sleep, Duration};

#[derive(Debug)]
enum Event {
    UserJoined(String),
    UserLeft(String),
    Message(String, String),  // (user, message)
}

#[tokio::main]
async fn main() {
    let (tx, rx) = mpsc::channel(32);
    let mut stream = ReceiverStream::new(rx);
    
    // Event producer
    tokio::spawn(async move {
        tx.send(Event::UserJoined("Alice".to_string())).await.unwrap();
        sleep(Duration::from_millis(100)).await;
        
        tx.send(Event::Message("Alice".to_string(), "Hello!".to_string())).await.unwrap();
        sleep(Duration::from_millis(100)).await;
        
        tx.send(Event::UserJoined("Bob".to_string())).await.unwrap();
        sleep(Duration::from_millis(100)).await;
        
        tx.send(Event::Message("Bob".to_string(), "Hi Alice!".to_string())).await.unwrap();
        sleep(Duration::from_millis(100)).await;
        
        tx.send(Event::UserLeft("Alice".to_string())).await.unwrap();
    });
    
    // Event processor
    while let Some(event) = stream.next().await {
        match event {
            Event::UserJoined(user) => {
                println!("📥 {} joined the chat", user);
            }
            Event::UserLeft(user) => {
                println!("📤 {} left the chat", user);
            }
            Event::Message(user, msg) => {
                println!("💬 {}: {}", user, msg);
            }
        }
    }
    
    println!("Chat closed");
}
```

Output:
```
📥 Alice joined the chat
💬 Alice: Hello!
📥 Bob joined the chat
💬 Bob: Hi Alice!
📤 Alice left the chat
Chat closed
```

## Advanced Pattern: Multiple Streams

You can merge multiple streams into one:

```rust
use tokio::sync::mpsc;
use tokio_stream::wrappers::ReceiverStream;
use tokio_stream::StreamExt;

#[tokio::main]
async fn main() {
    let (tx1, rx1) = mpsc::channel(32);
    let (tx2, rx2) = mpsc::channel(32);
    
    let stream1 = ReceiverStream::new(rx1);
    let stream2 = ReceiverStream::new(rx2);
    
    // Spawn two producers
    tokio::spawn(async move {
        for i in 0..3 {
            tx1.send(format!("Stream1: {}", i)).await.unwrap();
            tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
        }
    });
    
    tokio::spawn(async move {
        for i in 0..3 {
            tx2.send(format!("Stream2: {}", i)).await.unwrap();
            tokio::time::sleep(tokio::time::Duration::from_millis(150)).await;
        }
    });
    
    // Merge the streams
    let mut merged = stream1.merge(stream2);
    
    while let Some(value) = merged.next().await {
        println!("Got: {}", value);
    }
}
```

Output (order may vary due to timing):
```
Got: Stream1: 0
Got: Stream1: 1
Got: Stream2: 0
Got: Stream1: 2
Got: Stream2: 1
Got: Stream2: 2
```

## Key Differences: `while let Some` vs `select!`

### `while let Some` Pattern

```rust
while let Some(value) = stream.next().await {
    println!("Got: {}", value);
}
```

**Use when:**
- Processing one stream sequentially
- Order matters
- Simple iteration over values

### `select!` Pattern

```rust
loop {
    tokio::select! {
        Some(value1) = stream1.next() => {
            println!("Stream1: {}", value1);
        }
        Some(value2) = stream2.next() => {
            println!("Stream2: {}", value2);
        }
    }
}
```

**Use when:**
- Racing multiple streams
- Need to react to first available value
- Concurrent processing of multiple sources

## Best Practices

### 1. Always Use `mut` for Streams

```rust
// ✅ Correct: Stream needs to be mutable
let mut stream = ReceiverStream::new(rx);

// ❌ Error: Cannot call next() on immutable stream
let stream = ReceiverStream::new(rx);
stream.next().await;  // Compile error!
```

### 2. Don't Forget `.await`

```rust
// ❌ Error: next() returns a Future, not Option
while let Some(value) = stream.next() {
    // Won't compile
}

// ✅ Correct: Await the future
while let Some(value) = stream.next().await {
    // Works!
}
```

### 3. Handle Stream End Explicitly (Optional)

```rust
loop {
    match stream.next().await {
        Some(value) => {
            println!("Got: {}", value);
        }
        None => {
            println!("Stream ended - no more values");
            break;
        }
    }
}
```

### 4. Consider Error Handling

For streams that can produce errors:

```rust
use futures::TryStreamExt;

while let Some(result) = stream.try_next().await? {
    println!("Got: {}", result);
}
```

## Common Use Cases

1. **Network I/O**: Processing incoming connections or data
2. **Event loops**: Handling UI or system events
3. **Message processing**: Consuming from message queues
4. **Data pipelines**: Transform and process data streams
5. **Real-time updates**: WebSocket messages, database changes
6. **Log processing**: Reading and filtering log files

## Summary

The `while let Some` pattern with streams provides:

1. **Async iteration**: Process values as they arrive asynchronously
2. **Clean syntax**: Simple and readable code
3. **Automatic termination**: Loop exits when stream ends (returns `None`)
4. **Composability**: Can use stream combinators for complex processing
5. **Unified interface**: Works with any type implementing `Stream`

### Pattern Structure

```rust
// Convert channel to stream
let mut stream = ReceiverStream::new(rx);

// Iterate until stream ends
while let Some(value) = stream.next().await {
    // Process value
}

// Stream ended (channel closed)
```

This pattern is the async equivalent of iterating over collections with iterators, but designed for values that arrive over time from async sources like channels, network sockets, or file I/O operations.