# Selecting Stream Elements with `take` and `skip`

## Overview

This code demonstrates how to use the **`skip` and `take` combinators** to select specific elements from a stream. These combinators enable powerful stream slicing operations similar to array/iterator operations, allowing you to implement pagination, windowing, and subset selection in async data pipelines.

## Complete Code

```rust
use tokio_stream as stream;
use futures::StreamExt;

#[tokio::main]
async fn main() {
    // Skip first 3, then take next 4
    let stream = stream::iter(1..=10)
        .skip(3)
        .take(4);
    
    let results: Vec<i32> = stream.collect().await;
    println!("Results: {:?}", results); // Should be [4, 5, 6, 7]
}
```

## Cargo.toml

```toml
[package]
name = "stream-take-skip-example"
version = "0.1.0"
edition = "2021"

[dependencies]
tokio = { version = "1", features = ["full"] }
tokio-stream = "0.1"
futures = "0.3"
```

## Expected Output

```
Results: [4, 5, 6, 7]
```

## What are `skip` and `take`?

### The `skip` Combinator

Skips the first `n` elements in the stream and yields all elements after that.

**Function Signature:**
```rust
fn skip(self, n: usize) -> Skip<Self>
```

**Behavior:**
- Discards the first `n` items
- Yields all remaining items
- If stream has fewer than `n` items, yields nothing

### The `take` Combinator

Takes only the first `n` elements from the stream and then ends.

**Function Signature:**
```rust
fn take(self, n: usize) -> Take<Self>
```

**Behavior:**
- Yields the first `n` items
- Stops after `n` items (even if more are available)
- If stream has fewer than `n` items, yields all available

## How This Code Works

### Step 1: Create a Stream from Range

```rust
let stream = stream::iter(1..=10)
```

Creates a stream with values from 1 to 10 (inclusive):
```
Original Stream: [1, 2, 3, 4, 5, 6, 7, 8, 9, 10]
```

### Step 2: Apply `skip(3)`

```rust
    .skip(3)
```

Skips the first 3 elements:

```
Skip 3 elements:
╔═══╗ ╔═══╗ ╔═══╗
║ 1 ║ ║ 2 ║ ║ 3 ║ ← Discarded
╚═══╝ ╚═══╝ ╚═══╝

┌───┬───┬───┬───┬───┬───┬───┐
│ 4 │ 5 │ 6 │ 7 │ 8 │ 9 │10 │ ← Remaining
└───┴───┴───┴───┴───┴───┴───┘

After skip(3): [4, 5, 6, 7, 8, 9, 10]
```

### Step 3: Apply `take(4)`

```rust
    .take(4);
```

Takes only the first 4 elements from the remaining stream:

```
Take 4 elements:
┌───┬───┬───┬───┐
│ 4 │ 5 │ 6 │ 7 │ ← Kept
└───┴───┴───┴───┘

╔═══╗ ╔═══╗ ╔════╗
║ 8 ║ ║ 9 ║ ║ 10 ║ ← Discarded (not taken)
╚═══╝ ╚═══╝ ╚════╝

After take(4): [4, 5, 6, 7]
```

### Step 4: Collect Results

```rust
let results: Vec<i32> = stream.collect().await;
```

Collects the filtered stream into a vector: `[4, 5, 6, 7]`

### Step 5: Print Results

```rust
println!("Results: {:?}", results);
```

Prints: `Results: [4, 5, 6, 7]`

## Visual Data Flow

```
Original Stream
┌──────────────────────────────────────────────┐
│ 1, 2, 3, 4, 5, 6, 7, 8, 9, 10                │
└───────────────┬──────────────────────────────┘
                │
                ▼
           .skip(3)
                │
    ┌───────────┼───────────┐
    │  Skip first 3:        │
    │  ✗ 1 (skip)           │
    │  ✗ 2 (skip)           │
    │  ✗ 3 (skip)           │
    │  ✓ 4 (keep)           │
    │  ✓ 5 (keep)           │
    │  ✓ 6 (keep)           │
    │  ✓ 7 (keep)           │
    │  ✓ 8 (keep)           │
    │  ✓ 9 (keep)           │
    │  ✓ 10 (keep)          │
    └───────────┬───────────┘
                │
                ▼
      Stream: [4, 5, 6, 7, 8, 9, 10]
                │
                ▼
           .take(4)
                │
    ┌───────────┼───────────┐
    │  Take first 4:        │
    │  ✓ 4 (take)           │
    │  ✓ 5 (take)           │
    │  ✓ 6 (take)           │
    │  ✓ 7 (take)           │
    │  ✗ 8 (stop)           │
    │  ✗ 9 (stop)           │
    │  ✗ 10 (stop)          │
    └───────────┬───────────┘
                │
                ▼
      Final Stream: [4, 5, 6, 7]
                │
                ▼
          .collect().await
                │
                ▼
           Vec<i32>
        [4, 5, 6, 7]
```

## Comparison: Array Slicing vs Stream Operations

### Array Slicing (Synchronous)

```rust
let numbers = vec![1, 2, 3, 4, 5, 6, 7, 8, 9, 10];
let slice = &numbers[3..7];  // [4, 5, 6, 7]
println!("Results: {:?}", slice);
```

**Characteristics:**
- Direct memory access
- Instant operation
- Random access
- Works on in-memory data

### Stream Operations (Asynchronous)

```rust
let stream = tokio_stream::iter(1..=10)
    .skip(3)
    .take(4);

let results: Vec<i32> = stream.collect().await;
println!("Results: {:?}", results);
```

**Characteristics:**
- Sequential access
- Lazy evaluation
- Works on async sources
- Can handle infinite streams

## Common Patterns

### Pattern 1: Using `skip` Alone

```rust
use tokio_stream as stream;
use futures::StreamExt;

#[tokio::main]
async fn main() {
    let results: Vec<i32> = stream::iter(1..=10)
        .skip(5)  // Skip first 5
        .collect()
        .await;
    
    println!("Results: {:?}", results);
}
```

Output: `Results: [6, 7, 8, 9, 10]`

### Pattern 2: Using `take` Alone

```rust
use tokio_stream as stream;
use futures::StreamExt;

#[tokio::main]
async fn main() {
    let results: Vec<i32> = stream::iter(1..=10)
        .take(5)  // Take first 5
        .collect()
        .await;
    
    println!("Results: {:?}", results);
}
```

Output: `Results: [1, 2, 3, 4, 5]`

### Pattern 3: Pagination (Page 2, Size 3)

```rust
use tokio_stream as stream;
use futures::StreamExt;

#[tokio::main]
async fn main() {
    let page = 2;      // Page number (0-indexed)
    let page_size = 3; // Items per page
    
    let results: Vec<i32> = stream::iter(1..=10)
        .skip(page * page_size)  // Skip to page 2: skip 6 items
        .take(page_size)          // Take 3 items
        .collect()
        .await;
    
    println!("Page {}: {:?}", page, results);
}
```

Output: `Page 2: [7, 8, 9]`

### Pattern 4: Getting Last N Elements

```rust
use tokio_stream as stream;
use futures::StreamExt;

#[tokio::main]
async fn main() {
    let total = 10;
    let last_n = 3;
    
    let results: Vec<i32> = stream::iter(1..=10)
        .skip(total - last_n)  // Skip all but last 3
        .collect()
        .await;
    
    println!("Last {}: {:?}", last_n, results);
}
```

Output: `Last 3: [8, 9, 10]`

### Pattern 5: Window/Sliding Selection

```rust
use tokio_stream as stream;
use futures::StreamExt;

#[tokio::main]
async fn main() {
    // Get elements 3-5 (middle section)
    let results: Vec<i32> = stream::iter(1..=10)
        .skip(2)   // Skip first 2
        .take(3)   // Take next 3
        .collect()
        .await;
    
    println!("Window [3-5]: {:?}", results);
}
```

Output: `Window [3-5]: [3, 4, 5]`

## Practical Examples

### Example 1: Pagination System

```rust
use tokio_stream as stream;
use futures::StreamExt;

#[derive(Debug)]
struct Product {
    id: i32,
    name: String,
}

async fn get_page(page: usize, page_size: usize) -> Vec<Product> {
    // Simulate database with 50 products
    let all_products = (1..=50).map(|id| Product {
        id,
        name: format!("Product {}", id),
    });
    
    stream::iter(all_products)
        .skip(page * page_size)
        .take(page_size)
        .collect()
        .await
}

#[tokio::main]
async fn main() {
    let page_size = 10;
    
    for page in 0..3 {
        let products = get_page(page, page_size).await;
        println!("\n=== Page {} ===", page + 1);
        for product in products {
            println!("  {:?}", product);
        }
    }
}
```

Output:
```
=== Page 1 ===
  Product { id: 1, name: "Product 1" }
  Product { id: 2, name: "Product 2" }
  ...
  Product { id: 10, name: "Product 10" }

=== Page 2 ===
  Product { id: 11, name: "Product 11" }
  Product { id: 12, name: "Product 12" }
  ...
  Product { id: 20, name: "Product 20" }

=== Page 3 ===
  Product { id: 21, name: "Product 21" }
  Product { id: 22, name: "Product 22" }
  ...
  Product { id: 30, name: "Product 30" }
```

### Example 2: Processing Recent Records

```rust
use tokio_stream as stream;
use futures::StreamExt;

#[derive(Debug)]
struct LogEntry {
    id: i32,
    message: String,
}

#[tokio::main]
async fn main() {
    let log_entries = (1..=100).map(|id| LogEntry {
        id,
        message: format!("Log entry {}", id),
    });
    
    // Skip old logs, process only the last 10
    let recent_logs: Vec<LogEntry> = stream::iter(log_entries)
        .skip(90)   // Skip first 90
        .take(10)   // Take last 10
        .collect()
        .await;
    
    println!("Recent logs:");
    for log in recent_logs {
        println!("  {:?}", log);
    }
}
```

Output:
```
Recent logs:
  LogEntry { id: 91, message: "Log entry 91" }
  LogEntry { id: 92, message: "Log entry 92" }
  ...
  LogEntry { id: 100, message: "Log entry 100" }
```

### Example 3: Batch Processing

```rust
use tokio_stream as stream;
use futures::StreamExt;
use tokio::time::{sleep, Duration};

async fn process_batch(batch_num: usize, items: Vec<i32>) {
    println!("Processing batch {} with {} items", batch_num, items.len());
    sleep(Duration::from_millis(100)).await;
    println!("Batch {} complete", batch_num);
}

#[tokio::main]
async fn main() {
    let batch_size = 5;
    let total_items = 20;
    
    for batch_num in 0..(total_items / batch_size) {
        let batch: Vec<i32> = stream::iter(1..=total_items)
            .skip(batch_num * batch_size)
            .take(batch_size)
            .collect()
            .await;
        
        process_batch(batch_num, batch).await;
    }
}
```

Output:
```
Processing batch 0 with 5 items
Batch 0 complete
Processing batch 1 with 5 items
Batch 1 complete
Processing batch 2 with 5 items
Batch 2 complete
Processing batch 3 with 5 items
Batch 3 complete
```

### Example 4: Sampling Data

```rust
use tokio_stream as stream;
use futures::StreamExt;

#[tokio::main]
async fn main() {
    let data: Vec<i32> = (1..=100).collect();
    
    // Take every 10th element starting from the 5th
    let mut samples = Vec::new();
    
    for i in 0..10 {
        let sample: Vec<i32> = stream::iter(data.clone())
            .skip(5 + i * 10)  // Skip to position
            .take(1)            // Take 1 element
            .collect()
            .await;
        
        if let Some(&value) = sample.first() {
            samples.push(value);
        }
    }
    
    println!("Samples: {:?}", samples);
}
```

Output: `Samples: [6, 16, 26, 36, 46, 56, 66, 76, 86, 96]`

### Example 5: Removing Header and Footer

```rust
use tokio_stream as stream;
use futures::StreamExt;

#[tokio::main]
async fn main() {
    let lines = vec![
        "=== HEADER ===",
        "Data line 1",
        "Data line 2",
        "Data line 3",
        "Data line 4",
        "Data line 5",
        "=== FOOTER ===",
    ];
    
    let data_only: Vec<&str> = stream::iter(lines)
        .skip(1)                           // Skip header
        .take(lines.len() - 2)             // Exclude footer
        .collect()
        .await;
    
    println!("Data only:");
    for line in data_only {
        println!("  {}", line);
    }
}
```

Output:
```
Data only:
  Data line 1
  Data line 2
  Data line 3
  Data line 4
  Data line 5
```

## Edge Cases and Behavior

### Case 1: Skip More Than Available

```rust
let results: Vec<i32> = stream::iter(1..=5)
    .skip(10)  // Skip 10, but only 5 items exist
    .collect()
    .await;

println!("Results: {:?}", results);
// Output: Results: []
```

### Case 2: Take More Than Available

```rust
let results: Vec<i32> = stream::iter(1..=5)
    .take(10)  // Take 10, but only 5 items exist
    .collect()
    .await;

println!("Results: {:?}", results);
// Output: Results: [1, 2, 3, 4, 5]
```

### Case 3: Skip and Take More Than Available

```rust
let results: Vec<i32> = stream::iter(1..=5)
    .skip(3)   // Skip 3, leaving [4, 5]
    .take(10)  // Try to take 10, but only 2 left
    .collect()
    .await;

println!("Results: {:?}", results);
// Output: Results: [4, 5]
```

### Case 4: Skip Zero

```rust
let results: Vec<i32> = stream::iter(1..=5)
    .skip(0)  // Skip nothing
    .collect()
    .await;

println!("Results: {:?}", results);
// Output: Results: [1, 2, 3, 4, 5]
```

### Case 5: Take Zero

```rust
let results: Vec<i32> = stream::iter(1..=5)
    .take(0)  // Take nothing
    .collect()
    .await;

println!("Results: {:?}", results);
// Output: Results: []
```

## Combining with Other Combinators

### Skip, Filter, Take

```rust
use tokio_stream as stream;
use futures::StreamExt;

#[tokio::main]
async fn main() {
    let results: Vec<i32> = stream::iter(1..=20)
        .skip(5)                                              // [6..20]
        .filter(|x| futures::future::ready(x % 2 == 0))      // Evens only
        .take(5)                                              // First 5 evens
        .collect()
        .await;
    
    println!("Results: {:?}", results);
}
```

Output: `Results: [6, 8, 10, 12, 14]`

### Map, Skip, Take

```rust
let results: Vec<i32> = stream::iter(1..=20)
    .map(|x| x * 2)      // Double all
    .skip(3)             // Skip first 3 doubled
    .take(4)             // Take next 4
    .collect()
    .await;

// Results: [8, 10, 12, 14]
```

### Skip, Take, Then

```rust
use tokio::time::{sleep, Duration};

let results: Vec<i32> = stream::iter(1..=20)
    .skip(5)
    .take(3)
    .then(|x| async move {
        sleep(Duration::from_millis(10)).await;
        x * x
    })
    .collect()
    .await;

// Results: [36, 49, 64] (squares of 6, 7, 8)
```

## Performance Considerations

### Lazy Evaluation

```rust
// Stream not evaluated until collected
let stream = stream::iter(1..=1000000)
    .skip(500000)
    .take(10);

// Only processes 500,010 items (efficient early termination)
let results: Vec<i32> = stream.collect().await;
```

### Order of Operations Matters

```rust
// ✅ Better: Filter first (less work)
.filter(expensive_predicate)  // Reduces items
.skip(100)
.take(10)

// ❌ Worse: Skip/take first (more work)
.skip(100)
.take(10)
.filter(expensive_predicate)  // Filters only 10 items
```

## Comparison Table

| Operation | Original Stream | After `.skip(3)` | After `.take(4)` |
|-----------|----------------|------------------|------------------|
| **Input** | `[1,2,3,4,5,6,7,8,9,10]` | `[4,5,6,7,8,9,10]` | `[4,5,6,7]` |
| **Items Kept** | 10 | 7 | 4 |
| **Items Discarded** | 0 | 3 (front) | 3 (back) |

## Best Practices

### 1. Use for Stream Slicing

```rust
// ✅ Good: Clear intent for slicing
.skip(start_index)
.take(count)
```

### 2. Implement Pagination

```rust
fn get_page<T>(stream: impl Stream<Item = T>, page: usize, size: usize) 
    -> impl Stream<Item = T> 
{
    stream.skip(page * size).take(size)
}
```

### 3. Combine with Other Combinators

```rust
// Filter then paginate
.filter(predicate)
.skip(page * page_size)
.take(page_size)
```

### 4. Early Termination

```rust
// ✅ Good: Take limits processing
.take(100)  // Only processes 100 items max
.map(expensive_operation)

// ❌ Bad: All items processed
.map(expensive_operation)
.take(100)  // Processes all before taking
```

## Common Use Cases

1. **Pagination**: Loading pages of data from databases or APIs
2. **Windowing**: Processing specific sections of data streams
3. **Sampling**: Taking every Nth element or random samples
4. **Trimming**: Removing headers, footers, or metadata
5. **Batching**: Processing data in fixed-size chunks
6. **Rate Limiting**: Limiting number of items processed
7. **Testing**: Working with subsets of large datasets

## Summary

The `skip` and `take` combinators provide powerful stream slicing capabilities:

1. **`skip(n)`**: Discards first `n` elements, yields rest
2. **`take(n)`**: Yields first `n` elements, discards rest
3. **Chainable**: Combine for precise element selection
4. **Lazy**: Only processes necessary elements
5. **Safe**: Handles edge cases (empty streams, overflow)

### Basic Patterns

```rust
// Skip first N
.skip(n)

// Take first N
.take(n)

// Get elements from index A to B
.skip(A).take(B - A)

// Pagination (page P, size S)
.skip(P * S).take(S)
```

### When to Use

- **Pagination**: Divide large datasets into pages
- **Subsets**: Select specific ranges of data
- **Performance**: Process only needed elements
- **Windowing**: Implement sliding or fixed windows
- **Sampling**: Extract representative subsets

The combination of `skip` and `take` enables precise control over which stream elements are processed, making them essential tools for efficient async data processing in Rust.