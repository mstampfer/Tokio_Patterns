# Creating a Throttled Stream with `zip` and `interval`

## Overview

This code demonstrates how to create a **throttled stream** that emits values at a controlled rate. By combining a data stream with an interval timer using the `zip` combinator, you can pace the emission of values, implementing rate limiting, backpressure, or controlled data flow in async applications.

## Complete Code

```rust
use tokio_stream as stream;
use tokio_stream::wrappers::IntervalStream;
use futures::StreamExt;
use tokio::time::{interval, Duration, Instant};

#[tokio::main]
async fn main() {
    let start = Instant::now();
    
    let values = stream::iter(1..=5);
    let throttle = IntervalStream::new(interval(Duration::from_millis(100)));
    
    // Zip the values with the throttle interval
    let mut throttled = values.zip(throttle);
    
    while let Some((value, _)) = throttled.next().await {
        println!("Value {} at {:?}", value, start.elapsed());
    }
}
```

## Cargo.toml

```toml
[package]
name = "throttled-stream-example"
version = "0.1.0"
edition = "2021"

[dependencies]
tokio = { version = "1", features = ["full"] }
tokio-stream = "0.1"
futures = "0.3"
```

## Expected Output

```
Value 1 at ~100ms
Value 2 at ~200ms
Value 3 at ~300ms
Value 4 at ~400ms
Value 5 at ~500ms
```

The values are emitted exactly 100ms apart, regardless of how fast the source stream could produce them.

## What is Stream Throttling?

**Stream throttling** is the technique of controlling the rate at which values are emitted from a stream. Even if a stream can produce values very quickly, throttling ensures they are consumed at a controlled pace.

### Without Throttling

```rust
let values = stream::iter(1..=5);

while let Some(value) = values.next().await {
    println!("Value {} at {:?}", value, start.elapsed());
}
```

Output (instantaneous):
```
Value 1 at 0ms
Value 2 at 0ms
Value 3 at 0ms
Value 4 at 0ms
Value 5 at 0ms
```

### With Throttling

```rust
let throttled = values.zip(throttle);

while let Some((value, _)) = throttled.next().await {
    println!("Value {} at {:?}", value, start.elapsed());
}
```

Output (paced):
```
Value 1 at ~100ms
Value 2 at ~200ms
Value 3 at ~300ms
Value 4 at ~400ms
Value 5 at ~500ms
```

## How This Code Works

### Step 1: Create the Data Stream

```rust
let values = stream::iter(1..=5);
```

Creates a stream with values `[1, 2, 3, 4, 5]` that are immediately available.

**Stream characteristics:**
- All values ready instantly
- No inherent delay
- Would emit all values at once if consumed directly

### Step 2: Create the Throttle Timer

```rust
let throttle = IntervalStream::new(interval(Duration::from_millis(100)));
```

**What this does:**

1. **`interval(Duration::from_millis(100))`**: Creates a Tokio interval that fires every 100ms
2. **`IntervalStream::new(...)`**: Wraps the interval as a stream
3. **Result**: A stream that yields a tick every 100ms

**Interval behavior:**
- First tick happens immediately (at 0ms)
- Subsequent ticks every 100ms
- Infinite stream of ticks

### Step 3: Zip the Streams Together

```rust
let mut throttled = values.zip(throttle);
```

**The `zip` combinator:**
- Combines two streams into one
- Produces tuples of paired values: `(value_from_stream1, value_from_stream2)`
- Waits for BOTH streams to have a value ready
- Stops when either stream ends

**Visual representation:**
```
Values Stream:    [1,    2,    3,    4,    5]
                   ↓     ↓     ↓     ↓     ↓
                 wait  wait  wait  wait  wait
                   ↓     ↓     ↓     ↓     ↓
Throttle Stream:  [T,    T,    T,    T,    T]  (T = tick)
                   ↓     ↓     ↓     ↓     ↓
Zipped Stream:   [(1,T),(2,T),(3,T),(4,T),(5,T)]
```

**Key insight:** Since the throttle stream only produces values every 100ms, the zipped stream can only emit values at that same rate, effectively throttling the output.

### Step 4: Consume the Throttled Stream

```rust
while let Some((value, _)) = throttled.next().await {
    println!("Value {} at {:?}", value, start.elapsed());
}
```

**Execution:**
1. `throttled.next().await` waits for the next tuple
2. Destructures into `(value, _)` - we keep the value, ignore the tick
3. Prints the value and elapsed time
4. Repeats until stream ends

## Detailed Execution Timeline

```
Time    Event                                          Action
----    -----                                          ------
0ms     Program starts
        - values stream: has [1, 2, 3, 4, 5] ready
        - throttle stream: waiting for first tick
        
0ms     First interval tick occurs
        - throttle stream: yields tick
        - zip: both streams ready → emit (1, tick)
        - Print: "Value 1 at ~0ms"
        
        Next call to .next().await
        - values stream: has [2, 3, 4, 5] ready
        - throttle stream: waiting for next tick (100ms from now)
        - zip: waits...

100ms   Second interval tick occurs
        - throttle stream: yields tick
        - zip: both streams ready → emit (2, tick)
        - Print: "Value 2 at ~100ms"
        
        Next call to .next().await
        - values stream: has [3, 4, 5] ready
        - throttle stream: waiting for next tick
        - zip: waits...

200ms   Third interval tick occurs
        - throttle stream: yields tick
        - zip: both streams ready → emit (3, tick)
        - Print: "Value 3 at ~200ms"

300ms   Fourth interval tick occurs
        - zip: emit (4, tick)
        - Print: "Value 4 at ~300ms"

400ms   Fifth interval tick occurs
        - zip: emit (5, tick)
        - Print: "Value 5 at ~400ms"
        
        Next call to .next().await
        - values stream: exhausted (no more values)
        - zip: returns None (one stream ended)
        - Loop exits
```

## Visual Flow Diagram

```
┌─────────────────────────────────────────────────┐
│ Values Stream (Instant)                         │
│ [1, 2, 3, 4, 5] - All ready immediately        │
└────────────────┬────────────────────────────────┘
                 │
                 ▼
            .zip(throttle)
                 │
                 │ Waits for BOTH streams
                 │
┌────────────────┴────────────────────────────────┐
│ Throttle Stream (Interval)                      │
│                                                  │
│  0ms:   tick ──────────┐                       │
│ 100ms:  tick ──────────┤                       │
│ 200ms:  tick ──────────┤ Every 100ms           │
│ 300ms:  tick ──────────┤                       │
│ 400ms:  tick ──────────┘                       │
└─────────────────────────────────────────────────┘
                 │
                 ▼
      Zipped Stream Output
                 │
    ┌────────────┼────────────┐
    │                         │
    │  ~0ms:   (1, tick)      │
    │ ~100ms:  (2, tick)      │
    │ ~200ms:  (3, tick)      │
    │ ~300ms:  (4, tick)      │
    │ ~400ms:  (5, tick)      │
    │                         │
    └─────────────────────────┘
                 │
                 ▼
          .next().await
                 │
                 ▼
         Print each value
```

## Understanding `zip` Behavior

The `zip` combinator is the key to throttling. It combines two streams element-by-element:

```rust
Stream A: [a1, a2, a3, a4, a5]
Stream B: [b1, b2, b3, b4, b5]
Zipped:   [(a1,b1), (a2,b2), (a3,b3), (a4,b4), (a5,b5)]
```

**Critical behavior:**
- **Waits for both**: Must have values from BOTH streams before emitting a tuple
- **Pace of slowest**: Limited by whichever stream is slower
- **Ends with first**: Stops when either stream ends

### Throttling Mechanism

```
Fast Stream (instant):     [1][2][3][4][5]
                            ↓  ↓  ↓  ↓  ↓
                           wait wait wait wait
                            ↓  ↓  ↓  ↓  ↓
Slow Stream (100ms):       [T]   [T]   [T]   [T]   [T]
                            ↓     ↓     ↓     ↓     ↓
Result:                   (1,T) (2,T) (3,T) (4,T) (5,T)
                          0ms   100ms 200ms 300ms 400ms
```

The fast stream is effectively throttled to match the pace of the slow interval stream.

## Practical Examples

### Example 1: Rate-Limited API Calls

```rust
use tokio_stream as stream;
use tokio_stream::wrappers::IntervalStream;
use futures::StreamExt;
use tokio::time::{interval, Duration, Instant, sleep};

async fn call_api(id: i32) -> String {
    // Simulate API call
    sleep(Duration::from_millis(50)).await;
    format!("Response for ID {}", id)
}

#[tokio::main]
async fn main() {
    let start = Instant::now();
    
    // IDs to fetch
    let ids = vec![1, 2, 3, 4, 5, 6, 7, 8, 9, 10];
    
    // Throttle to 2 requests per second (500ms interval)
    let throttle = IntervalStream::new(interval(Duration::from_millis(500)));
    
    let mut throttled = stream::iter(ids).zip(throttle);
    
    println!("Starting rate-limited API calls (2 per second)...\n");
    
    while let Some((id, _)) = throttled.next().await {
        let response = call_api(id).await;
        println!("{} at {:?}", response, start.elapsed());
    }
}
```

Output:
```
Starting rate-limited API calls (2 per second)...

Response for ID 1 at ~50ms
Response for ID 2 at ~550ms
Response for ID 3 at ~1050ms
Response for ID 4 at ~1550ms
Response for ID 5 at ~2050ms
...
```

### Example 2: Progress Display with Throttling

```rust
use tokio_stream as stream;
use tokio_stream::wrappers::IntervalStream;
use futures::StreamExt;
use tokio::time::{interval, Duration, Instant};

#[tokio::main]
async fn main() {
    let start = Instant::now();
    
    // Process 50 items, show progress every 200ms
    let items = stream::iter(1..=50);
    let throttle = IntervalStream::new(interval(Duration::from_millis(200)));
    
    let mut throttled = items.zip(throttle);
    
    println!("Processing items...\n");
    
    while let Some((item, _)) = throttled.next().await {
        let progress = (item as f64 / 50.0) * 100.0;
        println!("[{:.1}%] Processing item {} at {:?}", 
                 progress, item, start.elapsed());
    }
    
    println!("\nAll items processed!");
}
```

Output:
```
Processing items...

[2.0%] Processing item 1 at ~0ms
[4.0%] Processing item 2 at ~200ms
[6.0%] Processing item 3 at ~400ms
[8.0%] Processing item 4 at ~600ms
...
[100.0%] Processing item 50 at ~9800ms

All items processed!
```

### Example 3: Controlled Database Batch Inserts

```rust
use tokio_stream as stream;
use tokio_stream::wrappers::IntervalStream;
use futures::StreamExt;
use tokio::time::{interval, Duration, sleep};

#[derive(Debug)]
struct Record {
    id: i32,
    data: String,
}

async fn insert_record(record: Record) {
    // Simulate database insert
    sleep(Duration::from_millis(30)).await;
    println!("Inserted: {:?}", record);
}

#[tokio::main]
async fn main() {
    let records = (1..=20).map(|id| Record {
        id,
        data: format!("Data for record {}", id),
    });
    
    // Throttle to 10 inserts per second
    let throttle = IntervalStream::new(interval(Duration::from_millis(100)));
    
    let mut throttled = stream::iter(records).zip(throttle);
    
    println!("Starting throttled database inserts...\n");
    
    while let Some((record, _)) = throttled.next().await {
        insert_record(record).await;
    }
    
    println!("\nAll records inserted!");
}
```

### Example 4: User Action Throttling (Debouncing)

```rust
use tokio_stream as stream;
use tokio_stream::wrappers::IntervalStream;
use futures::StreamExt;
use tokio::time::{interval, Duration, Instant};

#[tokio::main]
async fn main() {
    let start = Instant::now();
    
    // Simulate rapid user clicks
    let clicks = vec![
        "Click", "Click", "Click", "Click", "Click",
        "Click", "Click", "Click", "Click", "Click",
    ];
    
    // Only process one click per 300ms (throttle rapid clicks)
    let throttle = IntervalStream::new(interval(Duration::from_millis(300)));
    
    let mut throttled = stream::iter(clicks).zip(throttle);
    
    println!("User is clicking rapidly...\n");
    
    while let Some((action, _)) = throttled.next().await {
        println!("Processing: {} at {:?}", action, start.elapsed());
    }
}
```

Output:
```
User is clicking rapidly...

Processing: Click at ~0ms
Processing: Click at ~300ms
Processing: Click at ~600ms
Processing: Click at ~900ms
...
```

### Example 5: Network Packet Rate Limiting

```rust
use tokio_stream as stream;
use tokio_stream::wrappers::IntervalStream;
use futures::StreamExt;
use tokio::time::{interval, Duration, Instant};

#[derive(Debug)]
struct Packet {
    id: u32,
    data: Vec<u8>,
}

#[tokio::main]
async fn main() {
    let start = Instant::now();
    
    // Generate packets
    let packets = (1..=15).map(|id| Packet {
        id,
        data: vec![0u8; 100], // 100 bytes
    });
    
    // Rate limit: 1 packet per 50ms = 20 packets/second
    let throttle = IntervalStream::new(interval(Duration::from_millis(50)));
    
    let mut throttled = stream::iter(packets).zip(throttle);
    
    println!("Sending packets at 20 per second...\n");
    
    while let Some((packet, _)) = throttled.next().await {
        println!("Sent packet {} ({} bytes) at {:?}", 
                 packet.id, packet.data.len(), start.elapsed());
    }
}
```

## Different Throttling Strategies

### Strategy 1: Fixed Rate (Current Example)

```rust
let throttle = IntervalStream::new(interval(Duration::from_millis(100)));
let throttled = values.zip(throttle);
```

**Characteristics:**
- Constant rate (e.g., 10 per second)
- Simple and predictable
- Good for most rate limiting

### Strategy 2: Burst Then Throttle

```rust
let values = stream::iter(1..=20);
let throttle = IntervalStream::new(interval(Duration::from_millis(100)));

// Allow first 5 immediately, then throttle the rest
let throttled = values
    .enumerate()
    .zip(throttle)
    .map(|((i, value), _)| {
        if i < 5 {
            // No delay for first 5
            value
        } else {
            // Throttled after that
            value
        }
    });
```

### Strategy 3: Adaptive Throttling

```rust
let throttle_duration = Arc::new(Mutex::new(Duration::from_millis(100)));

// Adjust throttle_duration based on system load, error rates, etc.
```

## Comparison with Other Rate Limiting Techniques

### Using `sleep()` Directly

```rust
for value in values {
    process(value).await;
    sleep(Duration::from_millis(100)).await;  // Throttle
}
```

**Pros:** Simple
**Cons:** Blocking, not composable with streams

### Using `then()` with Delay

```rust
stream::iter(values)
    .then(|value| async move {
        sleep(Duration::from_millis(100)).await;
        value
    })
```

**Pros:** Works with streams
**Cons:** Delay happens AFTER processing

### Using `zip()` with Interval (Current)

```rust
stream::iter(values)
    .zip(IntervalStream::new(interval(Duration::from_millis(100))))
```

**Pros:** Composable, precise timing, delay happens BEFORE
**Cons:** Slightly more complex

## Performance Considerations

### Memory Efficiency

The `zip` approach is memory-efficient:
```rust
// ✅ Good: No buffering needed
values.zip(throttle)

// ❌ Potentially wasteful
values.collect().await; // Collects all first
```

### Timing Precision

Interval-based throttling is precise:
- Uses Tokio's timer wheel
- Accounts for processing time
- Maintains consistent rate

## Common Pitfalls

### Pitfall 1: First Tick is Immediate

```rust
// First value emitted immediately (at ~0ms)
let throttle = IntervalStream::new(interval(Duration::from_millis(100)));
```

**Solution:** Skip first tick if needed:
```rust
let throttle = IntervalStream::new(interval(Duration::from_millis(100)))
    .skip(1);
```

### Pitfall 2: Forgetting Tuple Destructuring

```rust
// ❌ Wrong: value is a tuple
while let Some(value) = throttled.next().await {
    println!("{}", value); // Error!
}

// ✅ Correct: destructure the tuple
while let Some((value, _)) = throttled.next().await {
    println!("{}", value);
}
```

### Pitfall 3: Throttle Slower Than Processing

```rust
// If processing takes 200ms but throttle is 100ms...
// Throttle has no effect (processing is already slower)
```

## Best Practices

### 1. Choose Appropriate Rate

```rust
// Consider:
// - External service limits (API rate limits)
// - System capacity
// - User experience

// Too fast: may overload
.zip(IntervalStream::new(interval(Duration::from_millis(10))))

// Too slow: poor throughput
.zip(IntervalStream::new(interval(Duration::from_secs(10))))

// Balanced
.zip(IntervalStream::new(interval(Duration::from_millis(100))))
```

### 2. Monitor and Adjust

```rust
// Log rate for monitoring
while let Some((value, _)) = throttled.next().await {
    metrics.increment("items_processed");
    process(value).await;
}
```

### 3. Consider Burst Allowance

```rust
// Allow initial burst, then throttle
let throttled = values
    .zip(throttle)
    .skip(1); // Skip immediate first tick
```

### 4. Handle Errors Gracefully

```rust
while let Some((value, _)) = throttled.next().await {
    match process(value).await {
        Ok(_) => { /* continue */ }
        Err(e) => {
            eprintln!("Error: {}", e);
            // Don't let one error stop throttling
        }
    }
}
```

## Summary

Throttling streams with `zip` and `interval` provides:

1. **Controlled rate**: Emit values at specific intervals
2. **Precise timing**: Uses Tokio's efficient timers
3. **Composable**: Works with other stream combinators
4. **Backpressure**: Naturally limits fast producers
5. **Simple pattern**: Easy to understand and implement

### Basic Pattern

```rust
let throttled = stream::iter(data)
    .zip(IntervalStream::new(interval(Duration::from_millis(rate_ms))));

while let Some((item, _)) = throttled.next().await {
    process(item).await;
}
```

### When to Use Throttling

- **Rate limiting**: Comply with API limits
- **Resource protection**: Prevent overwhelming systems
- **User experience**: Smooth progress updates
- **Backpressure**: Control fast data producers
- **Cost control**: Limit expensive operations

The `zip` + `interval` pattern is a powerful and elegant way to implement rate limiting in async Rust applications, providing precise control over data flow while maintaining clean, composable code.