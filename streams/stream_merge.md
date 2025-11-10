# Merging Multiple Streams with `StreamExt::merge`

## Overview

This code demonstrates how to use the **`merge` combinator** to combine two independent streams into a single unified stream. The `merge` operation is essential for handling multiple asynchronous data sources simultaneously, yielding values from all streams as they become available.

## Complete Code

```rust
use tokio::sync::mpsc;
use tokio_stream::wrappers::ReceiverStream;
use tokio_stream::StreamExt;
use tokio::time::{sleep, Duration};

#[tokio::main]
async fn main() {
    let (tx1, rx1) = mpsc::channel(32);
    let (tx2, rx2) = mpsc::channel(32);
    
    tokio::spawn(async move {
        for i in 0..3 {
            sleep(Duration::from_millis(50)).await;
            tx1.send(format!("Stream1: {}", i)).await.unwrap();
        }
    });
    
    tokio::spawn(async move {
        for i in 0..3 {
            sleep(Duration::from_millis(75)).await;
            tx2.send(format!("Stream2: {}", i)).await.unwrap();
        }
    });
    
    let stream1 = ReceiverStream::new(rx1);
    let stream2 = ReceiverStream::new(rx2);
    
    let mut merged = stream1.merge(stream2);
    
    while let Some(value) = merged.next().await {
        println!("{}", value);
    }
}
```

## Cargo.toml

```toml
[package]
name = "stream-merge-example"
version = "0.1.0"
edition = "2021"

[dependencies]
tokio = { version = "1", features = ["full"] }
tokio-stream = "0.1"
```

## Expected Output

```
Stream1: 0
Stream2: 0
Stream1: 1
Stream2: 1
Stream1: 2
Stream2: 2
```

**Note:** The exact order may vary slightly based on timing, but all 6 messages will appear. The key characteristic is that messages from both streams are interleaved based on when they arrive.

## What is `merge`?

The `merge` combinator combines two streams into a single stream that yields values from both sources. Values are emitted in the order they become available, not in any predetermined sequence.

### Function Signature

```rust
fn merge<U>(self, other: U) -> Merge<Self, U>
where
    U: Stream<Item = Self::Item>,
```

**Key characteristics:**
- Takes two streams with the same `Item` type
- Returns a new merged stream
- Yields items as soon as they're ready from either stream
- Non-deterministic ordering (depends on timing)
- Continues until both streams end

## How This Code Works

### Step 1: Create Two Channels

```rust
let (tx1, rx1) = mpsc::channel(32);
let (tx2, rx2) = mpsc::channel(32);
```

Creates two independent MPSC channels for two separate data sources.

### Step 2: Spawn Producer Tasks

```rust
tokio::spawn(async move {
    for i in 0..3 {
        sleep(Duration::from_millis(50)).await;
        tx1.send(format!("Stream1: {}", i)).await.unwrap();
    }
});

tokio::spawn(async move {
    for i in 0..3 {
        sleep(Duration::from_millis(75)).await;
        tx2.send(format!("Stream2: {}", i)).await.unwrap();
    }
});
```

**Producer 1:**
- Sends messages every 50ms
- Messages: "Stream1: 0", "Stream1: 1", "Stream1: 2"
- Timing: 50ms, 100ms, 150ms

**Producer 2:**
- Sends messages every 75ms
- Messages: "Stream2: 0", "Stream2: 1", "Stream2: 2"
- Timing: 75ms, 150ms, 225ms

### Step 3: Convert Receivers to Streams

```rust
let stream1 = ReceiverStream::new(rx1);
let stream2 = ReceiverStream::new(rx2);
```

Wraps the channel receivers as streams, enabling use of stream combinators.

### Step 4: Merge the Streams

```rust
let mut merged = stream1.merge(stream2);
```

**What happens:**
- Creates a new stream that polls both `stream1` and `stream2`
- Yields whichever value becomes available first
- Continues polling both streams until both end
- Type of merged stream: `Merge<ReceiverStream<String>, ReceiverStream<String>>`

### Step 5: Consume the Merged Stream

```rust
while let Some(value) = merged.next().await {
    println!("{}", value);
}
```

**Execution:**
1. Calls `merged.next().await` to get the next value
2. The merged stream checks both underlying streams
3. Returns the first available value
4. Prints the value
5. Repeats until both streams are exhausted

## Detailed Execution Timeline

```
Time    Stream1                Stream2                Merged Output
----    -------                -------                -------------
0ms     Waiting (50ms)         Waiting (75ms)         Waiting...

50ms    Sends "Stream1: 0" ───────────────────────> "Stream1: 0"
        Waiting (50ms)         Waiting (25ms)         

75ms    Waiting (25ms)         Sends "Stream2: 0" ──> "Stream2: 0"
                               Waiting (75ms)

100ms   Sends "Stream1: 1" ───────────────────────> "Stream1: 1"
        Waiting (50ms)         Waiting (50ms)

150ms   Sends "Stream1: 2" ──┐                     
        Done                  ├───────────────────> "Stream1: 2"
                              │ Sends "Stream2: 1" ─> "Stream2: 1"
                              │ Waiting (75ms)
                              
225ms                          Sends "Stream2: 2" ──> "Stream2: 2"
                               Done

        Both streams ended
        Merged stream ends
        Loop exits
```

## Visual Representation

### Stream Timeline Diagram

```
Stream1: ──50ms──┬─50ms──┬─50ms──┬──
                 │       │       │
               [0]     [1]     [2]

Stream2: ──75ms────────┬─75ms────────┬─75ms────────┬──
                       │             │             │
                     [0]           [1]           [2]

Merged:  ─────┬───┬────┬──┬────┬────┬──
              │   │    │  │    │    │
            [S1:0][S2:0][S1:1][S1:2,S2:1][S2:2]
```

### Data Flow Diagram

```
┌─────────────────────────┐
│ Producer Task 1         │
│ (50ms intervals)        │
└──────────┬──────────────┘
           │
           ▼
     ┌──────────┐
     │ Channel 1│
     └──────┬───┘
            │
            ▼
   ┌────────────────┐
   │   Stream1      │
   └────────┬───────┘
            │
            ├──────────┐
            │          │
            ▼          │
   ┌────────────────┐  │
   │     MERGE      │◄─┘
   └────────┬───────┘
            │
            ▼
   ┌────────────────┐
   │ Merged Stream  │
   └────────┬───────┘
            │
            ▼
       .next().await
            │
            ▼
      Print output

┌─────────────────────────┐
│ Producer Task 2         │
│ (75ms intervals)        │
└──────────┬──────────────┘
           │
           ▼
     ┌──────────┐
     │ Channel 2│
     └──────┬───┘
            │
            ▼
   ┌────────────────┐
   │   Stream2      │
   └────────┬───────┘
            │
            └──────────────┘
```

## Key Characteristics of `merge`

### 1. Non-Deterministic Ordering

The order of items depends on when they arrive:

```rust
// Order is based on timing, not stream priority
Stream1: [A at 50ms, B at 100ms]
Stream2: [X at 75ms, Y at 125ms]
Merged:  [A, X, B, Y]
```

### 2. Fair Polling

Both streams are polled fairly:
- No stream is prioritized over the other
- All available values are eventually yielded
- No starvation of either stream

### 3. Ends When Both End

The merged stream continues until BOTH input streams are exhausted:

```rust
Stream1: [A, B, C] (then closes)
Stream2: [X, Y, Z, W, Q] (then closes)
Merged:  [A, X, B, Y, C, Z, W, Q]
         ↑                     ↑
         First               Last (both closed)
```

### 4. Same Item Type Required

Both streams must produce the same type:

```rust
// ✅ Good: Both produce String
let merged = stream1.merge(stream2);

// ❌ Won't compile: Different types
let stream1: Stream<Item = i32> = ...;
let stream2: Stream<Item = String> = ...;
let merged = stream1.merge(stream2); // Error!
```

## Practical Examples

### Example 1: Merging Multiple Event Sources

```rust
use tokio::sync::mpsc;
use tokio_stream::wrappers::ReceiverStream;
use tokio_stream::StreamExt;
use tokio::time::{sleep, Duration};

#[derive(Debug)]
enum Event {
    Click(String),
    KeyPress(char),
    Scroll(i32),
}

#[tokio::main]
async fn main() {
    let (click_tx, click_rx) = mpsc::channel(32);
    let (key_tx, key_rx) = mpsc::channel(32);
    
    // Simulate click events
    tokio::spawn(async move {
        for i in 0..3 {
            sleep(Duration::from_millis(100)).await;
            click_tx.send(Event::Click(format!("Button{}", i))).await.unwrap();
        }
    });
    
    // Simulate keyboard events
    tokio::spawn(async move {
        for ch in ['a', 'b', 'c'] {
            sleep(Duration::from_millis(80)).await;
            key_tx.send(Event::KeyPress(ch)).await.unwrap();
        }
    });
    
    let click_stream = ReceiverStream::new(click_rx);
    let key_stream = ReceiverStream::new(key_rx);
    
    let mut events = click_stream.merge(key_stream);
    
    println!("Processing events:\n");
    while let Some(event) = events.next().await {
        println!("{:?}", event);
    }
}
```

Output:
```
Processing events:

KeyPress('a')
Click("Button0")
KeyPress('b')
KeyPress('c')
Click("Button1")
Click("Button2")
```

### Example 2: Combining Multiple API Data Sources

```rust
use tokio::sync::mpsc;
use tokio_stream::wrappers::ReceiverStream;
use tokio_stream::StreamExt;
use tokio::time::{sleep, Duration};

#[derive(Debug)]
struct DataPoint {
    source: String,
    value: i32,
}

async fn fetch_from_api1(tx: mpsc::Sender<DataPoint>) {
    for i in 1..=3 {
        sleep(Duration::from_millis(100)).await;
        tx.send(DataPoint {
            source: "API-1".to_string(),
            value: i * 10,
        }).await.unwrap();
    }
}

async fn fetch_from_api2(tx: mpsc::Sender<DataPoint>) {
    for i in 1..=3 {
        sleep(Duration::from_millis(150)).await;
        tx.send(DataPoint {
            source: "API-2".to_string(),
            value: i * 20,
        }).await.unwrap();
    }
}

#[tokio::main]
async fn main() {
    let (tx1, rx1) = mpsc::channel(32);
    let (tx2, rx2) = mpsc::channel(32);
    
    tokio::spawn(fetch_from_api1(tx1));
    tokio::spawn(fetch_from_api2(tx2));
    
    let stream1 = ReceiverStream::new(rx1);
    let stream2 = ReceiverStream::new(rx2);
    
    let mut merged = stream1.merge(stream2);
    
    println!("Aggregating data from multiple APIs:\n");
    while let Some(data) = merged.next().await {
        println!("Received from {}: {}", data.source, data.value);
    }
}
```

Output:
```
Aggregating data from multiple APIs:

Received from API-1: 10
Received from API-2: 20
Received from API-1: 20
Received from API-1: 30
Received from API-2: 40
Received from API-2: 60
```

### Example 3: Merging Log Streams

```rust
use tokio::sync::mpsc;
use tokio_stream::wrappers::ReceiverStream;
use tokio_stream::StreamExt;
use tokio::time::{sleep, Duration, Instant};

#[derive(Debug)]
struct LogEntry {
    timestamp: Duration,
    service: String,
    message: String,
}

#[tokio::main]
async fn main() {
    let start = Instant::now();
    
    let (service1_tx, service1_rx) = mpsc::channel(32);
    let (service2_tx, service2_rx) = mpsc::channel(32);
    
    // Service 1 logs
    tokio::spawn({
        let start = start.clone();
        async move {
            for i in 1..=3 {
                sleep(Duration::from_millis(70)).await;
                service1_tx.send(LogEntry {
                    timestamp: start.elapsed(),
                    service: "Auth".to_string(),
                    message: format!("User action {}", i),
                }).await.unwrap();
            }
        }
    });
    
    // Service 2 logs
    tokio::spawn({
        let start = start.clone();
        async move {
            for i in 1..=3 {
                sleep(Duration::from_millis(90)).await;
                service2_tx.send(LogEntry {
                    timestamp: start.elapsed(),
                    service: "Database".to_string(),
                    message: format!("Query {}", i),
                }).await.unwrap();
            }
        }
    });
    
    let stream1 = ReceiverStream::new(service1_rx);
    let stream2 = ReceiverStream::new(service2_rx);
    
    let mut logs = stream1.merge(stream2);
    
    println!("Unified log stream:\n");
    while let Some(log) = logs.next().await {
        println!("[{:?}] {}: {}", log.timestamp, log.service, log.message);
    }
}
```

Output:
```
Unified log stream:

[70ms] Auth: User action 1
[90ms] Database: Query 1
[140ms] Auth: User action 2
[180ms] Database: Query 2
[210ms] Auth: User action 3
[270ms] Database: Query 3
```

### Example 4: Merging Sensor Data

```rust
use tokio::sync::mpsc;
use tokio_stream::wrappers::ReceiverStream;
use tokio_stream::StreamExt;
use tokio::time::{sleep, Duration};

#[derive(Debug)]
struct SensorReading {
    sensor_id: String,
    temperature: f64,
}

#[tokio::main]
async fn main() {
    let (sensor1_tx, sensor1_rx) = mpsc::channel(32);
    let (sensor2_tx, sensor2_rx) = mpsc::channel(32);
    
    // Sensor 1 (living room)
    tokio::spawn(async move {
        for i in 0..5 {
            sleep(Duration::from_millis(100)).await;
            sensor1_tx.send(SensorReading {
                sensor_id: "Living Room".to_string(),
                temperature: 20.0 + (i as f64 * 0.5),
            }).await.unwrap();
        }
    });
    
    // Sensor 2 (bedroom)
    tokio::spawn(async move {
        for i in 0..5 {
            sleep(Duration::from_millis(120)).await;
            sensor2_tx.send(SensorReading {
                sensor_id: "Bedroom".to_string(),
                temperature: 18.0 + (i as f64 * 0.3),
            }).await.unwrap();
        }
    });
    
    let stream1 = ReceiverStream::new(sensor1_rx);
    let stream2 = ReceiverStream::new(sensor2_rx);
    
    let mut readings = stream1.merge(stream2);
    
    println!("Temperature monitoring:\n");
    while let Some(reading) = readings.next().await {
        println!("{}: {:.1}°C", reading.sensor_id, reading.temperature);
    }
}
```

Output:
```
Temperature monitoring:

Living Room: 20.0°C
Bedroom: 18.0°C
Living Room: 20.5°C
Bedroom: 18.3°C
Living Room: 21.0°C
Living Room: 21.5°C
Bedroom: 18.6°C
Living Room: 22.0°C
Bedroom: 18.9°C
Bedroom: 19.2°C
```

## Merging More Than Two Streams

To merge more than two streams, chain multiple `merge` calls:

```rust
let merged = stream1
    .merge(stream2)
    .merge(stream3)
    .merge(stream4);
```

Or use a macro for many streams:

```rust
use tokio_stream::StreamExt;

macro_rules! merge_all {
    ($first:expr, $($rest:expr),+) => {
        $first $(.merge($rest))+
    };
}

let merged = merge_all!(stream1, stream2, stream3, stream4);
```

### Example: Merging Multiple Sources

```rust
use tokio::sync::mpsc;
use tokio_stream::wrappers::ReceiverStream;
use tokio_stream::StreamExt;

#[tokio::main]
async fn main() {
    let (tx1, rx1) = mpsc::channel(10);
    let (tx2, rx2) = mpsc::channel(10);
    let (tx3, rx3) = mpsc::channel(10);
    
    // Spawn producers
    tokio::spawn(async move { tx1.send("A").await.unwrap(); });
    tokio::spawn(async move { tx2.send("B").await.unwrap(); });
    tokio::spawn(async move { tx3.send("C").await.unwrap(); });
    
    let merged = ReceiverStream::new(rx1)
        .merge(ReceiverStream::new(rx2))
        .merge(ReceiverStream::new(rx3));
    
    let values: Vec<&str> = merged.collect().await;
    println!("Collected: {:?}", values);
}
```

## Comparison: `merge` vs Other Combinators

### `merge` - Interleaved by Arrival

```rust
stream1.merge(stream2)
```

**Characteristics:**
- Yields items as they arrive
- Non-deterministic order
- Both streams polled concurrently

### `chain` - Sequential Concatenation

```rust
stream1.chain(stream2)
```

**Characteristics:**
- Yields all of stream1 first
- Then yields all of stream2
- Deterministic order
- Sequential, not concurrent

### `zip` - Paired Elements

```rust
stream1.zip(stream2)
```

**Characteristics:**
- Yields tuples of paired items
- Waits for both streams
- Deterministic pairing
- Stops when either ends

### Comparison Example

```rust
// Given:
Stream1: [A, B, C] (fast)
Stream2: [X, Y, Z] (slow)

// merge (interleaved)
Result: [A, X, B, Y, C, Z] (or any interleaving)

// chain (sequential)
Result: [A, B, C, X, Y, Z]

// zip (paired)
Result: [(A,X), (B,Y), (C,Z)]
```

## Performance Considerations

### Efficiency

`merge` is efficient:
- No buffering of items
- Streams polled only when needed
- Low memory overhead

### Fairness

Both streams are polled fairly:
```rust
// Neither stream starves
// Both get equal polling opportunities
stream1.merge(stream2)
```

### Backpressure

Backpressure is maintained:
```rust
// If consumer is slow, producers are slowed
// No unbounded buffering
```

## Common Pitfalls

### Pitfall 1: Different Item Types

```rust
// ❌ Won't compile
let stream1: Stream<Item = i32> = ...;
let stream2: Stream<Item = String> = ...;
let merged = stream1.merge(stream2);

// ✅ Convert to common type
let merged = stream1
    .map(|x| format!("{}", x))
    .merge(stream2);
```

### Pitfall 2: Assuming Order

```rust
// ❌ Bad: Assumes stream1 values come first
let merged = stream1.merge(stream2);
assert!(merged.next().await.unwrap().starts_with("Stream1"));

// ✅ Good: Don't assume order
let merged = stream1.merge(stream2);
// Process items regardless of source
```

### Pitfall 3: Not Awaiting `.next()`

```rust
// ❌ Won't compile
while let Some(value) = merged.next() {
    println!("{}", value);
}

// ✅ Correct
while let Some(value) = merged.next().await {
    println!("{}", value);
}
```

## Best Practices

### 1. Use Descriptive Types

```rust
#[derive(Debug)]
enum DataSource {
    Api1(String),
    Api2(String),
    Database(String),
}

let merged: Stream<Item = DataSource> = ...;
```

### 2. Handle Stream Completion

```rust
while let Some(value) = merged.next().await {
    process(value);
}
// Both streams have ended
println!("All sources exhausted");
```

### 3. Consider Using Enums for Tagged Data

```rust
enum Message {
    FromSource1(Data1),
    FromSource2(Data2),
}

let stream1 = source1.map(Message::FromSource1);
let stream2 = source2.map(Message::FromSource2);
let merged = stream1.merge(stream2);
```

### 4. Log or Track Sources

```rust
while let Some(value) = merged.next().await {
    metrics.increment(&format!("received_{}", value.source));
    process(value);
}
```

## Summary

The `merge` combinator provides powerful stream composition:

1. **Combines multiple streams**: Unifies separate data sources
2. **Concurrent polling**: All streams polled simultaneously
3. **Non-deterministic order**: Items yielded as they arrive
4. **Fair handling**: No stream starvation
5. **Ends when all end**: Continues until all sources exhausted

### Basic Pattern

```rust
let merged = stream1.merge(stream2);

while let Some(value) = merged.next().await {
    process(value);
}
```

### When to Use `merge`

- **Multiple data sources**: APIs, sensors, channels
- **Event aggregation**: Combining different event types
- **Log consolidation**: Unified logging from multiple services
- **Real-time updates**: Merging live data streams
- **Parallel data gathering**: Collecting from concurrent sources

The `merge` combinator is essential for building systems that need to handle multiple asynchronous data sources concurrently, providing a simple and efficient way to create unified stream interfaces in async Rust applications.