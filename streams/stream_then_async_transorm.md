# Applying Async Functions to Stream Elements with `then`

## Overview

This code demonstrates how to use the **`then` combinator** to apply an asynchronous function to each element in a stream. The `then` combinator is essential when you need to perform async operations (like network requests, database queries, or I/O) as part of your stream processing pipeline.

## Complete Code

```rust
use tokio_stream::StreamExt;
use tokio::time::{sleep, Duration};

#[tokio::main]
async fn main() {
    let mut stream = tokio_stream::iter(1..=5)
        .then(|x| async move {
            sleep(Duration::from_millis(10)).await;
            x * 2
        });
    
    while let Some(value) = stream.next().await {
        println!("Processed: {}", value);
    }
}
```

## Cargo.toml

```toml
[package]
name = "stream-then-async-example"
version = "0.1.0"
edition = "2021"

[dependencies]
tokio = { version = "1", features = ["full"] }
tokio-stream = "0.1"
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

The `then` combinator transforms each value in a stream by applying an **async function** (a function that returns a `Future`). It's the async equivalent of `map`, but specifically designed for transformations that involve `.await` operations.

### Function Signature

```rust
fn then<F, Fut>(self, f: F) -> Then<Self, F>
where
    F: FnMut(Self::Item) -> Fut,
    Fut: Future,
```

**Key characteristics:**
- Takes a closure that returns a `Future`
- Each transformation can contain `.await` operations
- Processes items sequentially by default
- Returns a new stream with transformed values

## How This Code Works

### Step 1: Create a Stream from Range

```rust
let mut stream = tokio_stream::iter(1..=5)
```

Creates a stream that yields integers from 1 to 5:
```
Stream: [1, 2, 3, 4, 5]
```

### Step 2: Apply Async Transformation with `then`

```rust
    .then(|x| async move {
        sleep(Duration::from_millis(10)).await;
        x * 2
    });
```

**What happens:**

1. **`|x|`**: For each value `x` in the stream
2. **`async move { ... }`**: Creates an async block that captures `x`
3. **`sleep(Duration::from_millis(10)).await`**: Simulates async work (waits 10ms)
4. **`x * 2`**: Returns the doubled value
5. The closure returns a `Future<Output = i32>`

**Important:** The `async move` block is crucial - it:
- Creates a future that can be awaited
- Moves `x` into the async block (necessary for closures)
- Allows `.await` operations inside

### Step 3: Consume the Stream

```rust
while let Some(value) = stream.next().await {
    println!("Processed: {}", value);
}
```

**Execution flow:**

1. Call `stream.next().await` to get the next value
2. This triggers the async transformation for that value
3. Wait for the transformation to complete (10ms + computation)
4. Receive the transformed value
5. Print the result
6. Repeat until stream is exhausted

## Visual Flow Diagram

```
Input Stream                Async Transformation              Output Stream
                           
1 ──────────────────────> async move {                    
                              sleep(10ms).await            
                              return 1 * 2  ─────────────> 2
                          }
                           
2 ──────────────────────> async move {
                              sleep(10ms).await
                              return 2 * 2  ─────────────> 4
                          }
                           
3 ──────────────────────> async move {
                              sleep(10ms).await
                              return 3 * 2  ─────────────> 6
                          }
                           
4 ──────────────────────> async move {
                              sleep(10ms).await
                              return 4 * 2  ─────────────> 8
                          }
                           
5 ──────────────────────> async move {
                              sleep(10ms).await
                              return 5 * 2  ─────────────> 10
                          }
```

## Execution Timeline (Sequential Processing)

```
Time    Event
----    -----
0ms     Request value 1 from stream
0ms     Start async block for 1
        └─> sleep(10ms) begins

10ms    Async block for 1 completes → returns 2
10ms    Print "Processed: 2"

10ms    Request value 2 from stream
10ms    Start async block for 2
        └─> sleep(10ms) begins

20ms    Async block for 2 completes → returns 4
20ms    Print "Processed: 4"

20ms    Request value 3 from stream
20ms    Start async block for 3
        └─> sleep(10ms) begins

30ms    Async block for 3 completes → returns 6
30ms    Print "Processed: 6"

30ms    Request value 4 from stream
30ms    Start async block for 4
        └─> sleep(10ms) begins

40ms    Async block for 4 completes → returns 8
40ms    Print "Processed: 8"

40ms    Request value 5 from stream
40ms    Start async block for 5
        └─> sleep(10ms) begins

50ms    Async block for 5 completes → returns 10
50ms    Print "Processed: 10"

50ms    Stream exhausted - exit loop
```

**Total time:** ~50ms (sequential processing, one at a time)

## Why Use `async move` Block?

### The Problem with Named Functions

```rust
async fn process(x: i32) -> i32 {
    sleep(Duration::from_millis(10)).await;
    x * 2
}

// ❌ This causes Unpin errors!
.then(|x| process(x))
```

**Issue:** Async functions return futures that are not `Unpin` by default, causing compiler errors.

### Solution 1: Inline `async move` Block (Recommended)

```rust
// ✅ This works!
.then(|x| async move {
    sleep(Duration::from_millis(10)).await;
    x * 2
})
```

**Benefits:**
- No `Unpin` issues
- No heap allocation
- Simple and clear
- Inline logic is visible

### Solution 2: Using `Box::pin`

```rust
async fn process(x: i32) -> i32 {
    sleep(Duration::from_millis(10)).await;
    x * 2
}

// ✅ This also works, but with overhead
.then(|x| Box::pin(process(x)))
```

**Benefits:**
- Can reuse named functions
- Good for complex logic

**Drawbacks:**
- Requires heap allocation
- Slight performance overhead

## Comparison: `map` vs `then`

### Using `map` (Synchronous)

```rust
let mut stream = tokio_stream::iter(1..=5)
    .map(|x| x * 2);  // ✅ For sync transformations
```

**Use when:**
- No `.await` needed
- Pure computation
- No I/O operations

### Using `then` (Asynchronous)

```rust
let mut stream = tokio_stream::iter(1..=5)
    .then(|x| async move {
        sleep(Duration::from_millis(10)).await;
        x * 2
    });  // ✅ For async transformations
```

**Use when:**
- Contains `.await` operations
- I/O operations (network, file, database)
- Async computations
- Need to spawn tasks

### Attempting `map` with Async (Won't Work)

```rust
// ❌ This won't compile!
let mut stream = tokio_stream::iter(1..=5)
    .map(|x| async move {
        sleep(Duration::from_millis(10)).await;
        x * 2
    });
// Error: stream contains futures, not values
```

## Practical Examples

### Example 1: Simulating API Calls

```rust
use tokio_stream::StreamExt;
use tokio::time::{sleep, Duration};

#[derive(Debug)]
struct User {
    id: i32,
    name: String,
}

#[tokio::main]
async fn main() {
    let user_ids = vec![1, 2, 3, 4, 5];
    
    let mut stream = tokio_stream::iter(user_ids)
        .then(|id| async move {
            // Simulate API call
            sleep(Duration::from_millis(100)).await;
            User {
                id,
                name: format!("User{}", id),
            }
        });
    
    println!("Fetching users...");
    while let Some(user) = stream.next().await {
        println!("Fetched: {:?}", user);
    }
}
```

Output:
```
Fetching users...
Fetched: User { id: 1, name: "User1" }
Fetched: User { id: 2, name: "User2" }
Fetched: User { id: 3, name: "User3" }
Fetched: User { id: 4, name: "User4" }
Fetched: User { id: 5, name: "User5" }
```

### Example 2: Database Queries

```rust
use tokio_stream::StreamExt;
use tokio::time::{sleep, Duration};

#[derive(Debug)]
struct Record {
    id: i32,
    data: String,
}

#[tokio::main]
async fn main() {
    let ids = vec![101, 102, 103, 104, 105];
    
    let mut stream = tokio_stream::iter(ids)
        .then(|id| async move {
            // Simulate database query
            sleep(Duration::from_millis(50)).await;
            Record {
                id,
                data: format!("Data for record {}", id),
            }
        });
    
    println!("Querying database...");
    while let Some(record) = stream.next().await {
        println!("{:?}", record);
    }
}
```

Output:
```
Querying database...
Record { id: 101, data: "Data for record 101" }
Record { id: 102, data: "Data for record 102" }
Record { id: 103, data: "Data for record 103" }
Record { id: 104, data: "Data for record 104" }
Record { id: 105, data: "Data for record 105" }
```

### Example 3: File Processing

```rust
use tokio_stream::StreamExt;
use tokio::time::{sleep, Duration};

#[tokio::main]
async fn main() {
    let filenames = vec!["file1.txt", "file2.txt", "file3.txt"];
    
    let mut stream = tokio_stream::iter(filenames)
        .then(|filename| async move {
            // Simulate reading file
            sleep(Duration::from_millis(30)).await;
            let size = filename.len() * 100; // Fake size
            (filename, size)
        });
    
    println!("Processing files...");
    while let Some((name, size)) = stream.next().await {
        println!("{}: {} bytes", name, size);
    }
}
```

Output:
```
Processing files...
file1.txt: 900 bytes
file2.txt: 900 bytes
file3.txt: 900 bytes
```

### Example 4: HTTP Requests with Error Handling

```rust
use tokio_stream::StreamExt;
use tokio::time::{sleep, Duration};

#[derive(Debug)]
enum ApiResult {
    Success(String),
    Error(String),
}

#[tokio::main]
async fn main() {
    let urls = vec![
        "https://api.example.com/users/1",
        "https://api.example.com/users/2",
        "https://api.example.com/users/3",
    ];
    
    let mut stream = tokio_stream::iter(urls)
        .then(|url| async move {
            // Simulate HTTP request
            sleep(Duration::from_millis(100)).await;
            
            // Simulate occasional failures
            if url.contains("users/2") {
                ApiResult::Error(format!("Failed to fetch {}", url))
            } else {
                ApiResult::Success(format!("Data from {}", url))
            }
        });
    
    while let Some(result) = stream.next().await {
        match result {
            ApiResult::Success(data) => println!("✓ {}", data),
            ApiResult::Error(err) => println!("✗ {}", err),
        }
    }
}
```

Output:
```
✓ Data from https://api.example.com/users/1
✗ Failed to fetch https://api.example.com/users/2
✓ Data from https://api.example.com/users/3
```

### Example 5: Multiple Async Operations

```rust
use tokio_stream::StreamExt;
use tokio::time::{sleep, Duration};

#[tokio::main]
async fn main() {
    let mut stream = tokio_stream::iter(1..=3)
        .then(|id| async move {
            // First async operation
            sleep(Duration::from_millis(20)).await;
            let data = format!("User{}", id);
            
            // Second async operation
            sleep(Duration::from_millis(10)).await;
            let enriched = format!("Enriched: {}", data);
            
            // Third async operation
            sleep(Duration::from_millis(10)).await;
            enriched.to_uppercase()
        });
    
    while let Some(result) = stream.next().await {
        println!("{}", result);
    }
}
```

Output:
```
ENRICHED: USER1
ENRICHED: USER2
ENRICHED: USER3
```

## Sequential vs Concurrent Processing

### Sequential Processing (Using `then`)

```rust
let mut stream = tokio_stream::iter(1..=5)
    .then(|x| async move {
        sleep(Duration::from_millis(100)).await;
        x * 2
    });

// Processes one at a time: 1, then 2, then 3, etc.
// Total time: 5 × 100ms = 500ms
```

**Characteristics:**
- One item processed at a time
- Preserves order
- Predictable timing
- Lower resource usage

### Concurrent Processing (Using `buffer_unordered`)

```rust
let mut stream = tokio_stream::iter(1..=5)
    .map(|x| async move {
        sleep(Duration::from_millis(100)).await;
        x * 2
    })
    .buffer_unordered(3);  // Process 3 at a time

// Processes up to 3 concurrently
// Total time: ~200ms
```

**Characteristics:**
- Multiple items processed simultaneously
- Order not preserved
- Faster throughput
- Higher resource usage

## Combining `then` with Other Combinators

### Filter then Process

```rust
use tokio_stream::StreamExt;
use tokio::time::{sleep, Duration};

#[tokio::main]
async fn main() {
    let results: Vec<i32> = tokio_stream::iter(1..=10)
        .filter(|x| futures::future::ready(x % 2 == 0))  // Only evens
        .then(|x| async move {
            sleep(Duration::from_millis(10)).await;
            x * x  // Square them
        })
        .collect()
        .await;
    
    println!("Squared evens: {:?}", results);
}
```

Output: `Squared evens: [4, 16, 36, 64, 100]`

### Process then Filter

```rust
let results: Vec<i32> = tokio_stream::iter(1..=10)
    .then(|x| async move {
        sleep(Duration::from_millis(10)).await;
        x * 2
    })
    .filter(|x| futures::future::ready(*x > 10))
    .collect()
    .await;

// Results: [12, 14, 16, 18, 20]
```

### Process then Take

```rust
let results: Vec<i32> = tokio_stream::iter(1..=100)
    .then(|x| async move {
        sleep(Duration::from_millis(10)).await;
        x * 2
    })
    .take(5)  // Only process first 5
    .collect()
    .await;

// Results: [2, 4, 6, 8, 10]
// Only 5 items processed (efficient early termination)
```

### Map, Then, Filter Chain

```rust
let results: Vec<i32> = tokio_stream::iter(1..=10)
    .map(|x| x + 10)                    // Sync: add 10
    .then(|x| async move {              // Async: multiply
        sleep(Duration::from_millis(5)).await;
        x * 2
    })
    .filter(|x| futures::future::ready(*x > 30))  // Keep > 30
    .collect()
    .await;

// Results: [32, 34, 36, 38, 40]
```

## Performance Considerations

### Lazy Evaluation

```rust
// Async blocks are not executed here
let stream = tokio_stream::iter(1..=1000000)
    .then(|x| async move {
        println!("Processing {}", x);  // Won't print yet!
        sleep(Duration::from_millis(10)).await;
        x * 2
    });

// Async blocks execute as values are pulled
let results: Vec<i32> = stream.take(3).collect().await;
// Only processes first 3 values
```

### Sequential Bottleneck

```rust
// ⚠️ Can be slow if each operation takes time
.then(|x| async move {
    sleep(Duration::from_secs(1)).await;  // 1 second each!
    x * 2
})
// For 100 items: 100 seconds total
```

**Solution:** Use `buffer_unordered` for concurrent processing when order doesn't matter.

## Common Patterns

### Pattern 1: Async Closure with Captured State

```rust
let multiplier = 3;

let mut stream = tokio_stream::iter(1..=5)
    .then(move |x| async move {
        sleep(Duration::from_millis(10)).await;
        x * multiplier  // Captures multiplier
    });
```

### Pattern 2: Error Handling

```rust
.then(|x| async move {
    match risky_operation(x).await {
        Ok(result) => Some(result),
        Err(e) => {
            eprintln!("Error processing {}: {}", x, e);
            None
        }
    }
})
.filter_map(|x| futures::future::ready(x))
```

### Pattern 3: Spawning Tasks

```rust
.then(|x| async move {
    tokio::spawn(async move {
        // Run in separate task
        expensive_computation(x).await
    })
    .await
    .unwrap()
})
```

### Pattern 4: Rate Limiting

```rust
use tokio::time::{sleep, Duration, Instant};

let rate_limit = Duration::from_millis(100);
let mut last_time = Instant::now();

.then(move |x| async move {
    let elapsed = last_time.elapsed();
    if elapsed < rate_limit {
        sleep(rate_limit - elapsed).await;
    }
    last_time = Instant::now();
    
    process(x).await
})
```

## Best Practices

### 1. Use `async move` for Closures

```rust
// ✅ Good: async move block
.then(|x| async move {
    process(x).await
})

// ❌ Bad: Named function (causes Unpin issues)
.then(|x| process(x))
```

### 2. Keep Logic Simple

```rust
// ✅ Good: Extract complex logic
async fn complex_processing(x: i32) -> Result<i32, Error> {
    // Complex logic here
}

.then(|x| async move {
    complex_processing(x).await.unwrap_or(0)
})

// ❌ Bad: Too much inline
.then(|x| async move {
    // 50 lines of complex logic...
})
```

### 3. Consider Concurrency Needs

```rust
// Sequential (when order matters)
.then(|x| async move { process(x).await })

// Concurrent (when order doesn't matter)
.map(|x| async move { process(x).await })
.buffer_unordered(10)
```

### 4. Handle Errors Appropriately

```rust
.then(|x| async move {
    match process(x).await {
        Ok(result) => result,
        Err(e) => {
            eprintln!("Failed: {}", e);
            default_value()
        }
    }
})
```

## Common Use Cases

1. **API Requests**: Fetching data from multiple endpoints
2. **Database Queries**: Loading records sequentially
3. **File I/O**: Reading or writing multiple files
4. **Image Processing**: Applying async transformations
5. **Data Enrichment**: Adding data from external sources
6. **Validation**: Async validation checks
7. **Caching**: Checking cache then fetching

## Summary

The `then` combinator is essential for applying async functions to stream elements:

1. **Async transformations**: Apply functions that use `.await`
2. **Sequential by default**: Processes one item at a time
3. **Preserves order**: Items processed in stream order
4. **Use `async move`**: Inline blocks avoid `Unpin` issues
5. **Composable**: Chains with other stream combinators

### Basic Pattern

```rust
let mut stream = tokio_stream::iter(items)
    .then(|item| async move {
        // Async operations here
        transform(item).await
    });

while let Some(result) = stream.next().await {
    // Process result
}
```

### When to Use `then`

- Transformation involves `.await` operations
- Network requests, database queries, file I/O
- Any async computation
- Need to preserve order of async results

The `then` combinator enables powerful async data processing pipelines, bridging synchronous data sources with asynchronous operations in a clean and composable way.