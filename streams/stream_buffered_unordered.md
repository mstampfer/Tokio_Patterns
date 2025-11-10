# Concurrent Stream Processing with `buffer_unordered`

## Overview

This code demonstrates how to use **`buffer_unordered`** to process stream items concurrently rather than sequentially. This powerful combinator enables parallel execution of async operations, dramatically improving throughput when operations are independent and can run simultaneously.

## Complete Code

```rust
use tokio_stream as stream;
use futures::StreamExt;
use tokio::time::{sleep, Duration, Instant};

async fn slow_operation(x: i32) -> i32 {
    sleep(Duration::from_millis(100)).await;
    x * 2
}

#[tokio::main]
async fn main() {
    let start = Instant::now();
    
    let results: Vec<i32> = stream::iter(1..=5)
        .map(|x| async move { slow_operation(x).await })
        .buffer_unordered(3) // Process up to 3 at a time
        .collect()
        .await;
    
    println!("Results: {:?}", results);
    println!("Time: {:?}", start.elapsed());
}
```

## Cargo.toml

```toml
[package]
name = "stream-buffer-unordered-example"
version = "0.1.0"
edition = "2021"

[dependencies]
tokio = { version = "1", features = ["full"] }
tokio-stream = "0.1"
futures = "0.3"
```

## Expected Output

```
Results: [2, 4, 6, 8, 10]
Time: ~200ms
```

**Important:** The order of results may vary (e.g., `[2, 6, 4, 8, 10]`) because `buffer_unordered` yields results as they complete, not in the original order.

## What is `buffer_unordered`?

The `buffer_unordered` combinator allows multiple futures in a stream to execute concurrently, yielding results in the order they complete rather than the order they were created.

### Function Signature

```rust
fn buffer_unordered(self, n: usize) -> BufferUnordered<Self>
where
    Self::Item: Future,
```

**Parameters:**
- `n`: Maximum number of futures to execute concurrently (buffer size)

**Behavior:**
- Executes up to `n` futures simultaneously
- Yields results as soon as they complete
- Does NOT preserve order (results may be out of sequence)
- Provides significant performance improvement for I/O-bound operations

## How This Code Works

### Step 1: Create a Stream

```rust
let start = Instant::now();

let results: Vec<i32> = stream::iter(1..=5)
```

Creates a stream with values `[1, 2, 3, 4, 5]` and records the start time.

### Step 2: Map to Futures

```rust
    .map(|x| async move { slow_operation(x).await })
```

**What happens:**
- For each value `x`, creates a future that will call `slow_operation(x)`
- The `async move` block captures `x` and returns a future
- At this point, futures are NOT executed yet (lazy evaluation)
- Stream now contains futures, not values

**Important:** Without `buffer_unordered`, these futures would execute sequentially.

### Step 3: Apply `buffer_unordered(3)`

```rust
    .buffer_unordered(3) // Process up to 3 at a time
```

**What happens:**
- Executes up to 3 futures concurrently
- As soon as one completes, starts the next one
- Yields results immediately as they complete (no waiting for order)
- Maintains a "window" of 3 active futures

### Step 4: Collect Results

```rust
    .collect()
    .await;
```

Collects all results into a `Vec<i32>`.

### Step 5: Print Results and Timing

```rust
println!("Results: {:?}", results);
println!("Time: {:?}", start.elapsed());
```

Shows the results and total execution time.

## Sequential vs Concurrent Execution

### Sequential Processing (Without `buffer_unordered`)

```rust
let results: Vec<i32> = stream::iter(1..=5)
    .then(|x| async move { slow_operation(x).await })
    .collect()
    .await;

// Time: ~500ms (5 items × 100ms each)
```

**Timeline:**
```
Time    Activity
----    --------
0ms     Start processing item 1
100ms   Item 1 completes (result: 2)
        Start processing item 2
200ms   Item 2 completes (result: 4)
        Start processing item 3
300ms   Item 3 completes (result: 6)
        Start processing item 4
400ms   Item 4 completes (result: 8)
        Start processing item 5
500ms   Item 5 completes (result: 10)
        ALL DONE

Total: 500ms
```

### Concurrent Processing (With `buffer_unordered(3)`)

```rust
let results: Vec<i32> = stream::iter(1..=5)
    .map(|x| async move { slow_operation(x).await })
    .buffer_unordered(3)
    .collect()
    .await;

// Time: ~200ms
```

**Timeline:**
```
Time    Activity                              Active Futures
----    --------                              --------------
0ms     Start items 1, 2, 3 concurrently     [1, 2, 3]

100ms   Item 2 completes (result: 4) ✓       [1, 3]
        Start item 4                          [1, 3, 4]

100ms   Item 1 completes (result: 2) ✓       [3, 4]
        Start item 5                          [3, 4, 5]

150ms   Item 3 completes (result: 6) ✓       [4, 5]

200ms   Item 4 completes (result: 8) ✓       [5]
        Item 5 completes (result: 10) ✓      []
        ALL DONE

Total: ~200ms (2.5× faster!)
```

**Key insight:** With buffer size 3, we process items in batches:
- Batch 1: Items 1, 2, 3 (parallel)
- Batch 2: Items 4, 5 (parallel)

## Visual Execution Flow

### Sequential Execution (for comparison)

```
Item 1: ████████████ (100ms) → 2
Item 2:             ████████████ (100ms) → 4
Item 3:                         ████████████ (100ms) → 6
Item 4:                                     ████████████ (100ms) → 8
Item 5:                                                 ████████████ (100ms) → 10
        └───────────────────────────────────────────────────────────┘
                            500ms total
```

### Concurrent Execution with `buffer_unordered(3)`

```
Item 1: ████████████ (100ms) → 2
Item 2: ████████████ (100ms) → 4
Item 3: ████████████ (100ms) → 6
Item 4:             ████████████ (100ms) → 8
Item 5:             ████████████ (100ms) → 10
        └────────────────────┘
             ~200ms total
             
Buffer: [1, 2, 3] → [4, 5] → done
```

## Detailed Step-by-Step Execution

```
Step 1: Buffer starts, pulls 3 items
  └─> Futures for 1, 2, 3 start executing concurrently

Step 2: After ~100ms, some futures complete
  └─> Let's say item 2 finishes first
  └─> Yield result: 4
  └─> Buffer has space, start future for item 4

Step 3: More completions
  └─> Item 1 finishes
  └─> Yield result: 2
  └─> Start future for item 5

Step 4: Remaining futures complete
  └─> Item 3 finishes → Yield: 6
  └─> Item 4 finishes → Yield: 8
  └─> Item 5 finishes → Yield: 10

Step 5: Stream exhausted
  └─> All results yielded: [4, 2, 6, 8, 10] (unordered!)
  └─> collect() returns Vec: [4, 2, 6, 8, 10]
```

## Order Considerations

### `buffer_unordered` - Results in Completion Order

```rust
.buffer_unordered(3)
// Results: [4, 2, 6, 8, 10] (or any order)
```

**Characteristics:**
- Results yielded as they complete
- Fastest overall throughput
- Order not preserved
- Use when order doesn't matter

### `buffered` - Results in Original Order

```rust
.buffered(3)
// Results: [2, 4, 6, 8, 10] (always in order)
```

**Characteristics:**
- Results yielded in original order
- May wait for slower futures
- Slightly slower than `buffer_unordered`
- Use when order matters

## Practical Examples

### Example 1: Concurrent API Calls

```rust
use tokio_stream as stream;
use futures::StreamExt;
use tokio::time::{sleep, Duration, Instant};

async fn fetch_user(id: i32) -> String {
    // Simulate API call with variable latency
    let latency = Duration::from_millis((id as u64 % 3 + 1) * 50);
    sleep(latency).await;
    format!("User{}", id)
}

#[tokio::main]
async fn main() {
    let start = Instant::now();
    let user_ids = vec![1, 2, 3, 4, 5, 6, 7, 8, 9, 10];
    
    let users: Vec<String> = stream::iter(user_ids)
        .map(|id| async move { fetch_user(id).await })
        .buffer_unordered(5) // Fetch 5 users concurrently
        .collect()
        .await;
    
    println!("Fetched {} users in {:?}", users.len(), start.elapsed());
    println!("Users: {:?}", users);
}
```

Output:
```
Fetched 10 users in ~300ms
Users: ["User2", "User1", "User3", "User5", "User4", ...]
```

**Without concurrency:** Would take ~1500ms (sum of all latencies)
**With concurrency:** Takes ~300ms (parallelized)

### Example 2: Concurrent Database Queries

```rust
use tokio_stream as stream;
use futures::StreamExt;
use tokio::time::{sleep, Duration, Instant};

#[derive(Debug)]
struct Record {
    id: i32,
    data: String,
}

async fn query_database(id: i32) -> Record {
    // Simulate database query
    sleep(Duration::from_millis(100)).await;
    Record {
        id,
        data: format!("Data for record {}", id),
    }
}

#[tokio::main]
async fn main() {
    let start = Instant::now();
    let ids = vec![101, 102, 103, 104, 105, 106, 107, 108];
    
    let records: Vec<Record> = stream::iter(ids)
        .map(|id| async move { query_database(id).await })
        .buffer_unordered(4) // Query 4 at a time
        .collect()
        .await;
    
    println!("Queried {} records in {:?}", records.len(), start.elapsed());
    for record in records {
        println!("  {:?}", record);
    }
}
```

Output:
```
Queried 8 records in ~200ms
  Record { id: 102, data: "Data for record 102" }
  Record { id: 101, data: "Data for record 101" }
  ...
```

### Example 3: Concurrent File Processing

```rust
use tokio_stream as stream;
use futures::StreamExt;
use tokio::time::{sleep, Duration, Instant};

async fn process_file(filename: &str) -> (String, usize) {
    // Simulate file processing
    sleep(Duration::from_millis(150)).await;
    let size = filename.len() * 1000; // Fake size
    (filename.to_string(), size)
}

#[tokio::main]
async fn main() {
    let start = Instant::now();
    let files = vec![
        "file1.txt", "file2.txt", "file3.txt", 
        "file4.txt", "file5.txt", "file6.txt",
    ];
    
    let results: Vec<(String, usize)> = stream::iter(files)
        .map(|filename| async move { process_file(filename).await })
        .buffer_unordered(3) // Process 3 files at a time
        .collect()
        .await;
    
    println!("Processed {} files in {:?}", results.len(), start.elapsed());
    for (name, size) in results {
        println!("  {}: {} bytes", name, size);
    }
}
```

Output:
```
Processed 6 files in ~300ms
  file2.txt: 9000 bytes
  file1.txt: 9000 bytes
  file3.txt: 9000 bytes
  ...
```

### Example 4: Error Handling with Concurrent Processing

```rust
use tokio_stream as stream;
use futures::StreamExt;
use tokio::time::{sleep, Duration};

async fn risky_operation(x: i32) -> Result<i32, String> {
    sleep(Duration::from_millis(50)).await;
    if x % 3 == 0 {
        Err(format!("Failed on {}", x))
    } else {
        Ok(x * 2)
    }
}

#[tokio::main]
async fn main() {
    let results: Vec<Result<i32, String>> = stream::iter(1..=10)
        .map(|x| async move { risky_operation(x).await })
        .buffer_unordered(4)
        .collect()
        .await;
    
    for result in results {
        match result {
            Ok(value) => println!("Success: {}", value),
            Err(e) => println!("Error: {}", e),
        }
    }
}
```

Output:
```
Success: 2
Success: 4
Error: Failed on 3
Success: 8
Success: 10
Error: Failed on 6
...
```

### Example 5: Progress Tracking

```rust
use tokio_stream as stream;
use futures::StreamExt;
use tokio::time::{sleep, Duration};
use std::sync::{Arc, Mutex};

async fn process_item(id: i32, completed: Arc<Mutex<usize>>) -> i32 {
    sleep(Duration::from_millis(100)).await;
    
    let mut count = completed.lock().unwrap();
    *count += 1;
    let current = *count;
    drop(count);
    
    println!("Completed {}/10", current);
    id * 2
}

#[tokio::main]
async fn main() {
    let completed = Arc::new(Mutex::new(0));
    
    let results: Vec<i32> = stream::iter(1..=10)
        .map(|id| {
            let completed = Arc::clone(&completed);
            async move { process_item(id, completed).await }
        })
        .buffer_unordered(3)
        .collect()
        .await;
    
    println!("\nAll results: {:?}", results);
}
```

Output:
```
Completed 1/10
Completed 2/10
Completed 3/10
Completed 4/10
...
Completed 10/10

All results: [4, 6, 2, 10, 8, ...]
```

## Choosing the Buffer Size

### Small Buffer (e.g., 2)

```rust
.buffer_unordered(2)
```

**Pros:**
- Lower memory usage
- Fewer concurrent connections/resources
- Better for rate-limited APIs

**Cons:**
- Slower overall throughput
- More sequential behavior

### Medium Buffer (e.g., 5-10)

```rust
.buffer_unordered(5)
```

**Pros:**
- Good balance of concurrency and resource usage
- Suitable for most use cases

**Cons:**
- May not maximize throughput for fast operations

### Large Buffer (e.g., 50-100)

```rust
.buffer_unordered(50)
```

**Pros:**
- Maximum throughput
- Best for fast operations with many items

**Cons:**
- High memory usage
- May overwhelm external services
- Risk of connection/resource exhaustion

## Performance Comparison

```rust
use tokio_stream as stream;
use futures::StreamExt;
use tokio::time::{sleep, Duration, Instant};

async fn benchmark(concurrency: usize) {
    let start = Instant::now();
    
    let _: Vec<i32> = stream::iter(1..=20)
        .map(|x| async move {
            sleep(Duration::from_millis(100)).await;
            x * 2
        })
        .buffer_unordered(concurrency)
        .collect()
        .await;
    
    println!("Concurrency {}: {:?}", concurrency, start.elapsed());
}

#[tokio::main]
async fn main() {
    benchmark(1).await;   // ~2000ms (sequential)
    benchmark(2).await;   // ~1000ms
    benchmark(5).await;   // ~400ms
    benchmark(10).await;  // ~200ms
    benchmark(20).await;  // ~100ms (all at once)
}
```

Output:
```
Concurrency 1: ~2000ms
Concurrency 2: ~1000ms
Concurrency 5: ~400ms
Concurrency 10: ~200ms
Concurrency 20: ~100ms
```

## Common Patterns

### Pattern 1: Rate Limiting

```rust
use tokio::time::{sleep, Duration};

// Limit to 10 concurrent requests
.map(|x| async move {
    let result = api_call(x).await;
    sleep(Duration::from_millis(100)).await; // Rate limit
    result
})
.buffer_unordered(10)
```

### Pattern 2: Retry Logic

```rust
async fn with_retry<F, Fut, T>(f: F) -> T
where
    F: Fn() -> Fut,
    Fut: Future<Output = Result<T, Error>>,
{
    for attempt in 1..=3 {
        match f().await {
            Ok(result) => return result,
            Err(e) if attempt < 3 => {
                sleep(Duration::from_millis(100)).await;
                continue;
            }
            Err(e) => panic!("Failed after 3 attempts"),
        }
    }
    unreachable!()
}

.map(|x| async move {
    with_retry(|| api_call(x)).await
})
.buffer_unordered(5)
```

### Pattern 3: Progress Reporting

```rust
.map(|x| async move {
    let result = process(x).await;
    println!("Processed item {}", x);
    result
})
.buffer_unordered(3)
```

## Comparison: Different Concurrent Processing Methods

| Method | Order Preserved | Concurrency | Use Case |
|--------|----------------|-------------|----------|
| **`then`** | ✅ Yes | ❌ No (sequential) | Order matters, no parallelism needed |
| **`buffered`** | ✅ Yes | ✅ Yes | Order matters, want concurrency |
| **`buffer_unordered`** | ❌ No | ✅ Yes | Order doesn't matter, max throughput |

### Example Comparison

```rust
// Sequential (order preserved, no concurrency)
.then(|x| async move { process(x).await })
// Time: 500ms, Order: [2, 4, 6, 8, 10]

// Concurrent (order preserved)
.map(|x| async move { process(x).await })
.buffered(3)
// Time: 200ms, Order: [2, 4, 6, 8, 10]

// Concurrent (order NOT preserved)
.map(|x| async move { process(x).await })
.buffer_unordered(3)
// Time: 200ms, Order: [4, 2, 6, 10, 8]
```

## Best Practices

### 1. Choose Appropriate Buffer Size

```rust
// Consider:
// - Available resources (connections, memory)
// - External service limits
// - Nature of operations (I/O vs CPU)

// Conservative
.buffer_unordered(5)

// Aggressive
.buffer_unordered(50)
```

### 2. Handle Errors Appropriately

```rust
.map(|x| async move {
    match risky_operation(x).await {
        Ok(result) => Some(result),
        Err(e) => {
            eprintln!("Error: {}", e);
            None
        }
    }
})
.buffer_unordered(10)
.filter_map(|x| async move { x })
```

### 3. Use Only When Order Doesn't Matter

```rust
// ✅ Good: Order doesn't matter
fetch_user_profiles().buffer_unordered(10)

// ❌ Bad: Order matters
process_sequential_steps().buffer_unordered(10)
// Use .buffered(10) instead
```

### 4. Monitor Resource Usage

```rust
// Be careful with buffer size
// Too large = resource exhaustion
// Too small = underutilized concurrency

// Start conservative, measure, adjust
.buffer_unordered(10)
```

## Common Pitfalls

### Pitfall 1: Assuming Order

```rust
// ❌ Bad: Assumes results are in order
let results = stream.buffer_unordered(5).collect().await;
assert_eq!(results[0], first_expected); // May fail!

// ✅ Good: Don't assume order
let mut results = stream.buffer_unordered(5).collect().await;
results.sort(); // Sort if order needed
```

### Pitfall 2: Buffer Too Large

```rust
// ❌ Bad: May exhaust connections
.buffer_unordered(1000) // Too many!

// ✅ Good: Reasonable limit
.buffer_unordered(20)
```

### Pitfall 3: Forgetting `async move`

```rust
// ❌ Won't work with buffer_unordered
.map(|x| slow_operation(x))

// ✅ Correct: Returns futures
.map(|x| async move { slow_operation(x).await })
```

## Summary

The `buffer_unordered` combinator enables efficient concurrent processing:

1. **Concurrent execution**: Process multiple items simultaneously
2. **Unordered results**: Yields as completions occur (fastest throughput)
3. **Configurable concurrency**: Control with buffer size parameter
4. **Significant speedup**: Ideal for I/O-bound operations
5. **Resource efficient**: Limits concurrent operations to prevent overload

### Basic Pattern

```rust
let results: Vec<T> = tokio_stream::iter(items)
    .map(|item| async move {
        async_operation(item).await
    })
    .buffer_unordered(concurrency_limit)
    .collect()
    .await;
```

### When to Use `buffer_unordered`

- **I/O-bound operations**: API calls, database queries, file operations
- **Independent tasks**: Operations don't depend on each other
- **Order doesn't matter**: Results can be processed in any order
- **Throughput matters**: Want maximum speed
- **Multiple items**: Have many items to process

The `buffer_unordered` combinator is essential for building high-performance async applications that process multiple independent operations efficiently.