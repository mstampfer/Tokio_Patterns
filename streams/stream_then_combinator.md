# Async Stream Transformations with the `then` Combinator

## Overview

This code demonstrates how to use the **`then` combinator** to perform asynchronous transformations on stream values. While `map` is used for synchronous transformations, `then` is essential when your transformation involves async operations like network requests, database queries, or any `.await` operations.

## Complete Code

```rust
use tokio_stream as stream;
use futures::StreamExt;
use tokio::time::{sleep, Duration};

async fn process(x: i32) -> i32 {
    sleep(Duration::from_millis(10)).await;
    x * 2
}

#[tokio::main]
async fn main() {
    let mut stream = stream::iter(1..=5)
        .then(|x| process(x));
    
    while let Some(value) = stream.next().await {
        println!("Processed: {}", value);
    }
}
```

## Cargo.toml

```toml
[package]
name = "stream-then-example"
version = "0.1.0"
edition = "2021"

[dependencies]
tokio = { version = "1", features = ["full"] }
tokio-stream = "0.1"
futures = "0.3"
```

## Expected Output

```
Processed: 2
Processed: 4
Processed: 6
Processed: 8
Processed: 10
```

## What is the `then` Combinator?

The `then` combinator transforms each value in a stream by applying an **async function** to it. It's the async equivalent of `map`.

### Function Signature

```rust
fn then<F, Fut>(self, f: F) -> Then<Self, F>
where
    F: FnMut(Self::Item) -> Fut,
    Fut: Future,
```

**Key differences from `map`:**
- The closure must return a `Future`
- Each transformation can contain `.await` operations
- Transformations execute asynchronously

## How This Code Works

### Step 1: Define Async Processing Function

```rust
async fn process(x: i32) -> i32 {
    sleep(Duration::from_millis(10)).await;
    x * 2
}
```

**What it does:**
- Takes an integer `x`
- Simulates async work by sleeping for 10ms
- Returns the doubled value

This represents any async operation: database query, API call, file I/O, etc.

### Step 2: Create Stream with `then`

```rust
let mut stream = stream::iter(1..=5)
    .then(|x| process(x));
```

**Execution flow:**
1. `stream::iter(1..=5)` creates a stream of values `[1, 2, 3, 4, 5]`
2. `.then(|x| process(x))` applies the async `process` function to each value
3. Each value is processed asynchronously when pulled from the stream

### Step 3: Consume the Stream

```rust
while let Some(value) = stream.next().await {
    println!("Processed: {}", value);
}
```

**What happens:**
1. `stream.next().await` pulls the next value
2. If a value exists, it's already been processed by `process()`
3. Print the processed value
4. Repeat until stream is exhausted

## Visual Flow Diagram

```
Input Stream                Async Processing              Output Stream
                           
1 ──────────────────────>  process(1)                    
                           │ sleep(10ms)                 
                           │ return 1 * 2  ──────────>   2
                           
2 ──────────────────────>  process(2)
                           │ sleep(10ms)
                           │ return 2 * 2  ──────────>   4
                           
3 ──────────────────────>  process(3)
                           │ sleep(10ms)
                           │ return 3 * 2  ──────────>   6
                           
4 ──────────────────────>  process(4)
                           │ sleep(10ms)
                           │ return 4 * 2  ──────────>   8
                           
5 ──────────────────────>  process(5)
                           │ sleep(10ms)
                           │ return 5 * 2  ──────────>   10
```

## Execution Timeline

```
Time    Event
----    -----
0ms     Pull value 1 from stream
0ms     Start process(1) - sleep begins
10ms    process(1) completes → returns 2
10ms    Print "Processed: 2"

10ms    Pull value 2 from stream
10ms    Start process(2) - sleep begins
20ms    process(2) completes → returns 4
20ms    Print "Processed: 4"

20ms    Pull value 3 from stream
20ms    Start process(3) - sleep begins
30ms    process(3) completes → returns 6
30ms    Print "Processed: 6"

30ms    Pull value 4 from stream
30ms    Start process(4) - sleep begins
40ms    process(4) completes → returns 8
40ms    Print "Processed: 8"

40ms    Pull value 5 from stream
40ms    Start process(5) - sleep begins
50ms    process(5) completes → returns 10
50ms    Print "Processed: 10"

50ms    Stream exhausted - exit loop
```

**Total execution time:** ~50ms (sequential processing)

## Key Difference: `map` vs `then`

### Using `map` (Synchronous)

```rust
let mut stream = stream::iter(1..=5)
    .map(|x| x * 2);  // ✅ Works for sync transformations
```

**Use when:**
- Transformation is synchronous (no `.await`)
- No I/O operations
- Pure computation

### Using `then` (Asynchronous)

```rust
let mut stream = stream::iter(1..=5)
    .then(|x| async move {
        sleep(Duration::from_millis(10)).await;
        x * 2
    });  // ✅ Works for async transformations
```

**Use when:**
- Transformation involves `.await`
- I/O operations (network, file, database)
- Async computations

### Attempting `map` with Async (Won't Work)

```rust
// ❌ This won't compile!
let mut stream = stream::iter(1..=5)
    .map(|x| async move {
        sleep(Duration::from_millis(10)).await;
        x * 2
    });
// Error: map expects a value, not a Future
```

## Comparison Table

| Feature | `map` | `then` |
|---------|-------|--------|
| **Closure returns** | Value `T` | Future `impl Future<Output = T>` |
| **Async operations** | ❌ No | ✅ Yes |
| **Use `.await`** | ❌ No | ✅ Yes |
| **Performance** | Faster (no async overhead) | Slower (async overhead) |
| **Use case** | Simple transformations | I/O, network, async work |

## Practical Examples

### Example 1: Simulating API Calls

```rust
use tokio_stream as stream;
use futures::StreamExt;
use tokio::time::{sleep, Duration};

async fn fetch_user_data(id: i32) -> String {
    // Simulate network request
    sleep(Duration::from_millis(100)).await;
    format!("User data for ID {}", id)
}

#[tokio::main]
async fn main() {
    let user_ids = vec![1, 2, 3, 4, 5];
    
    let mut stream = stream::iter(user_ids)
        .then(|id| fetch_user_data(id));
    
    while let Some(data) = stream.next().await {
        println!("{}", data);
    }
}
```

Output:
```
User data for ID 1
User data for ID 2
User data for ID 3
User data for ID 4
User data for ID 5
```

### Example 2: Database Queries

```rust
use tokio_stream as stream;
use futures::StreamExt;
use tokio::time::{sleep, Duration};

#[derive(Debug)]
struct Record {
    id: i32,
    value: String,
}

async fn query_database(id: i32) -> Record {
    // Simulate database query
    sleep(Duration::from_millis(50)).await;
    Record {
        id,
        value: format!("Record {}", id),
    }
}

#[tokio::main]
async fn main() {
    let ids = vec![101, 102, 103];
    
    let mut stream = stream::iter(ids)
        .then(|id| query_database(id));
    
    while let Some(record) = stream.next().await {
        println!("{:?}", record);
    }
}
```

Output:
```
Record { id: 101, value: "Record 101" }
Record { id: 102, value: "Record 102" }
Record { id: 103, value: "Record 103" }
```

### Example 3: File Operations

```rust
use tokio_stream as stream;
use futures::StreamExt;
use tokio::time::{sleep, Duration};

async fn read_file_size(filename: &str) -> (String, usize) {
    // Simulate file I/O
    sleep(Duration::from_millis(20)).await;
    (filename.to_string(), filename.len() * 100) // Fake size
}

#[tokio::main]
async fn main() {
    let files = vec!["file1.txt", "file2.txt", "file3.txt"];
    
    let mut stream = stream::iter(files)
        .then(|filename| async move {
            read_file_size(filename).await
        });
    
    while let Some((name, size)) = stream.next().await {
        println!("{}: {} bytes", name, size);
    }
}
```

Output:
```
file1.txt: 900 bytes
file2.txt: 900 bytes
file3.txt: 900 bytes
```

### Example 4: Error Handling with Results

```rust
use tokio_stream as stream;
use futures::StreamExt;
use tokio::time::{sleep, Duration};

async fn risky_operation(x: i32) -> Result<i32, String> {
    sleep(Duration::from_millis(10)).await;
    if x % 2 == 0 {
        Ok(x * 2)
    } else {
        Err(format!("Cannot process odd number: {}", x))
    }
}

#[tokio::main]
async fn main() {
    let mut stream = stream::iter(1..=5)
        .then(|x| risky_operation(x));
    
    while let Some(result) = stream.next().await {
        match result {
            Ok(value) => println!("Success: {}", value),
            Err(e) => println!("Error: {}", e),
        }
    }
}
```

Output:
```
Error: Cannot process odd number: 1
Success: 4
Error: Cannot process odd number: 3
Success: 8
Error: Cannot process odd number: 5
```

## Sequential vs Concurrent Processing

### Sequential (Using `then`)

```rust
let mut stream = stream::iter(1..=5)
    .then(|x| async move {
        sleep(Duration::from_millis(100)).await;
        x * 2
    });

// Processes one at a time: 1, then 2, then 3, etc.
// Total time: 500ms
```

### Concurrent (Using `buffer_unordered`)

```rust
let mut stream = stream::iter(1..=5)
    .map(|x| async move {
        sleep(Duration::from_millis(100)).await;
        x * 2
    })
    .buffer_unordered(3);  // Process 3 at a time

// Processes up to 3 concurrently
// Total time: ~200ms
```

## Combining `then` with Other Combinators

### Filter then Process

```rust
use tokio_stream as stream;
use futures::StreamExt;
use tokio::time::{sleep, Duration};

async fn expensive_computation(x: i32) -> i32 {
    sleep(Duration::from_millis(50)).await;
    x * x
}

#[tokio::main]
async fn main() {
    let results: Vec<i32> = stream::iter(1..=10)
        .filter(|x| futures::future::ready(x % 2 == 0))  // Only evens
        .then(|x| expensive_computation(x))               // Process async
        .collect()
        .await;
    
    println!("Results: {:?}", results);
}
```

Output: `Results: [4, 16, 36, 64, 100]`

### Process then Take

```rust
let results: Vec<i32> = stream::iter(1..=100)
    .then(|x| async move {
        sleep(Duration::from_millis(10)).await;
        x * 2
    })
    .take(5)  // Only process first 5
    .collect()
    .await;

// Results: [2, 4, 6, 8, 10]
```

### Process then Filter

```rust
let results: Vec<i32> = stream::iter(1..=10)
    .then(|x| async move {
        sleep(Duration::from_millis(10)).await;
        x * 2
    })
    .filter(|x| futures::future::ready(x > 10))
    .collect()
    .await;

// Results: [12, 14, 16, 18, 20]
```

## Advanced Pattern: Closure with Multiple Async Calls

```rust
use tokio_stream as stream;
use futures::StreamExt;
use tokio::time::{sleep, Duration};

async fn fetch_data(id: i32) -> i32 {
    sleep(Duration::from_millis(20)).await;
    id * 10
}

async fn enrich_data(data: i32) -> String {
    sleep(Duration::from_millis(10)).await;
    format!("Enriched: {}", data)
}

#[tokio::main]
async fn main() {
    let mut stream = stream::iter(1..=3)
        .then(|id| async move {
            let data = fetch_data(id).await;
            let enriched = enrich_data(data).await;
            enriched
        });
    
    while let Some(result) = stream.next().await {
        println!("{}", result);
    }
}
```

Output:
```
Enriched: 10
Enriched: 20
Enriched: 30
```

## Performance Considerations

### Sequential Processing (Default)

```rust
// Each item processed one after another
let mut stream = stream::iter(1..=5)
    .then(|x| async move {
        sleep(Duration::from_millis(100)).await;
        x * 2
    });

// Time: 5 × 100ms = 500ms
```

### Concurrent Processing with `buffer_unordered`

```rust
// Multiple items processed simultaneously
let mut stream = stream::iter(1..=5)
    .map(|x| async move {
        sleep(Duration::from_millis(100)).await;
        x * 2
    })
    .buffer_unordered(5);  // All 5 at once

// Time: ~100ms (all concurrent)
```

**Note:** Use `buffer_unordered` when:
- Operations are independent
- Order of results doesn't matter
- You want to maximize throughput

## Common Patterns

### Pattern 1: Inline Async Closure

```rust
.then(|x| async move {
    // Async operations here
    x * 2
})
```

### Pattern 2: Named Async Function

```rust
async fn process(x: i32) -> i32 {
    // Async operations here
    x * 2
}

.then(|x| process(x))
```

### Pattern 3: Async Closure with Error Handling

```rust
.then(|x| async move {
    match risky_operation(x).await {
        Ok(result) => result,
        Err(e) => {
            eprintln!("Error: {}", e);
            0  // Default value
        }
    }
})
```

## Best Practices

### 1. Use `then` for Async, `map` for Sync

```rust
// ✅ Good: map for sync
.map(|x| x * 2)

// ✅ Good: then for async
.then(|x| async move {
    fetch_data(x).await
})

// ❌ Bad: then for simple sync operations (unnecessary overhead)
.then(|x| async move { x * 2 })
```

### 2. Consider Concurrency Needs

```rust
// Sequential: when order matters or operations must be serial
.then(|x| process(x))

// Concurrent: when operations are independent
.map(|x| process(x))
.buffer_unordered(10)
```

### 3. Handle Errors Appropriately

```rust
.then(|x| async move {
    match process(x).await {
        Ok(result) => Some(result),
        Err(e) => {
            eprintln!("Failed to process {}: {}", x, e);
            None
        }
    }
})
.filter_map(|x| futures::future::ready(x))
```

### 4. Keep Closures Focused

```rust
// ✅ Good: Extract complex logic to named functions
async fn complex_processing(x: i32) -> Result<i32, Error> {
    // Complex logic here
}

.then(|x| complex_processing(x))

// ❌ Bad: Too much logic in closure
.then(|x| async move {
    // Many lines of complex logic...
})
```

## Common Use Cases

1. **API/HTTP Requests**: Fetching data from multiple endpoints
2. **Database Queries**: Loading records by ID
3. **File I/O**: Reading or writing multiple files
4. **Async Computation**: CPU-intensive work in spawned tasks
5. **Rate Limiting**: Adding delays between operations
6. **Retry Logic**: Retrying failed async operations
7. **Caching**: Checking cache then fetching if missing

## Summary

The `then` combinator is essential for async stream processing:

1. **Async transformations**: Apply async functions to stream items
2. **Sequential by default**: Processes one item at a time
3. **Flexible**: Can contain multiple `.await` calls
4. **Composable**: Chains with other stream combinators
5. **Type-safe**: Maintains type information through transformations

### When to Use `then`

- Transformation involves `.await` operations
- Network requests, database queries, file I/O
- Any async computation
- Need to preserve order of async results

### Basic Pattern

```rust
let mut stream = tokio_stream::iter(items)
    .then(|item| async_transform(item));

while let Some(result) = stream.next().await {
    // Process result
}
```

The `then` combinator bridges synchronous data sources with asynchronous processing, enabling powerful async data pipelines in Rust.