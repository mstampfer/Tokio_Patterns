# Creating Streams from Iterators with `tokio_stream::iter`

## Overview

This code demonstrates how to convert a synchronous iterator (like a `Vec`) into an asynchronous **Stream** using `tokio_stream::iter`. This bridge between synchronous and asynchronous code is essential for integrating existing collections and iterators into async pipelines.

## Complete Code

```rust
use tokio_stream;
use futures::StreamExt;

#[tokio::main]
async fn main() {
    let values = vec![1, 2, 3, 4, 5];
    let mut stream = tokio_stream::iter(values);
    
    while let Some(value) = stream.next().await {
        println!("Value: {}", value);
    }
}
```

### Cargo.toml

```toml
[package]
name = "stream-iter-example"
version = "0.1.0"
edition = "2021"

[dependencies]
tokio = { version = "1", features = ["full"] }
tokio-stream = "0.1"
futures = "0.3"
```

### Expected Output

```
Value: 1
Value: 2
Value: 3
Value: 4
Value: 5
```

## What is `tokio_stream::iter`?

`tokio_stream::iter` is a function that:

1. **Takes any iterator** (anything implementing `IntoIterator`)
2. **Returns a Stream** that yields the same values asynchronously
3. **Provides async interface** to synchronous data
4. **Enables stream combinators** to be used on regular collections

### Function Signature

```rust
pub fn iter<I>(i: I) -> Iter<I::IntoIter>
where
    I: IntoIterator,
```

## Why Convert Iterators to Streams?

### Synchronous Iterator

```rust
let values = vec![1, 2, 3, 4, 5];
for value in values {
    println!("Value: {}", value);
}
```

**Characteristics:**
- Blocking iteration
- Synchronous processing only
- Cannot mix with async operations
- No async combinators

### Async Stream

```rust
let values = vec![1, 2, 3, 4, 5];
let mut stream = tokio_stream::iter(values);

while let Some(value) = stream.next().await {
    println!("Value: {}", value);
}
```

**Characteristics:**
- Non-blocking iteration (yields to runtime)
- Can mix with other async operations
- Access to stream combinators
- Uniform interface with other async sources

## How This Code Works

### Step 1: Create a Vector

```rust
let values = vec![1, 2, 3, 4, 5];
```

A standard Rust vector containing integers 1 through 5.

### Step 2: Convert to Stream

```rust
let mut stream = tokio_stream::iter(values);
```

**What happens:**
1. `tokio_stream::iter()` takes ownership of the vector
2. Creates a `Stream` that will yield each element
3. The stream implements the `Stream` trait
4. Values are yielded one at a time when polled

**Important:** The vector is moved into the stream. If you need to keep the original, use:
```rust
let mut stream = tokio_stream::iter(values.clone());
// or
let mut stream = tokio_stream::iter(&values);  // For references
```

### Step 3: Iterate Over Stream

```rust
while let Some(value) = stream.next().await {
    println!("Value: {}", value);
}
```

**Execution flow:**

1. **`stream.next()`**: Returns a `Future<Output = Option<i32>>`
2. **`.await`**: Pauses here, allows runtime to handle other tasks
3. **`Some(value)`**: Pattern matches when a value is available
4. **Prints**: `"Value: {value}"`
5. **Repeats**: Until stream is exhausted
6. **`None`**: When no more values, loop exits

## Complete Code

```rust
use tokio_stream;
use futures::StreamExt;

#[tokio::main]
async fn main() {
    let values = vec![1, 2, 3, 4, 5];
    let mut stream = tokio_stream::iter(values);
    
    while let Some(value) = stream.next().await {
        println!("Value: {}", value);
    }
}
```

## Cargo.toml Setup

```toml
[package]
name = "stream-iter-example"
version = "0.1.0"
edition = "2021"

[dependencies]
tokio = { version = "1", features = ["full"] }
tokio-stream = "0.1"
futures = "0.3"
```

## Expected Output

```
Value: 1
Value: 2
Value: 3
Value: 4
Value: 5
```

## Visual Representation

```
Synchronous World          tokio_stream::iter()          Async World
                                                    
Vec<i32>                                            Stream<Item = i32>
[1, 2, 3, 4, 5]  ────────────────────────────>    
                        Conversion                     
                                                    ┌──> next().await → Some(1)
Iterator                                            ├──> next().await → Some(2)
  ↓                                                 ├──> next().await → Some(3)
 1, 2, 3, 4, 5                                      ├──> next().await → Some(4)
                                                    ├──> next().await → Some(5)
                                                    └──> next().await → None
```

## Different Ways to Create Streams from Iterators

### 1. From Vector (Owned)

```rust
let values = vec![1, 2, 3, 4, 5];
let mut stream = tokio_stream::iter(values);  // Moves values

while let Some(value) = stream.next().await {
    println!("Value: {}", value);
}
// values is no longer available here
```

### 2. From References

```rust
let values = vec![1, 2, 3, 4, 5];
let mut stream = tokio_stream::iter(&values);  // Borrows values

while let Some(value) = stream.next().await {
    println!("Value: {}", value);
}
// values is still available here
println!("Original: {:?}", values);
```

### 3. From Range

```rust
let mut stream = tokio_stream::iter(0..10);

while let Some(value) = stream.next().await {
    println!("Value: {}", value);
}
```

Output: `Value: 0` through `Value: 9`

### 4. From Array

```rust
let values = [10, 20, 30, 40, 50];
let mut stream = tokio_stream::iter(values);

while let Some(value) = stream.next().await {
    println!("Value: {}", value);
}
```

### 5. From HashMap

```rust
use std::collections::HashMap;

let mut map = HashMap::new();
map.insert("a", 1);
map.insert("b", 2);
map.insert("c", 3);

let mut stream = tokio_stream::iter(map);

while let Some((key, value)) = stream.next().await {
    println!("{}: {}", key, value);
}
```

## Practical Example: Processing with Async Operations

Here's where converting to a stream becomes powerful - you can use async operations in the processing:

```rust
use tokio_stream;
use futures::StreamExt;
use tokio::time::{sleep, Duration};

async fn process_value(value: i32) -> i32 {
    // Simulate async operation (e.g., API call, database query)
    sleep(Duration::from_millis(100)).await;
    value * 2
}

#[tokio::main]
async fn main() {
    let values = vec![1, 2, 3, 4, 5];
    let mut stream = tokio_stream::iter(values);
    
    while let Some(value) = stream.next().await {
        let result = process_value(value).await;
        println!("Processed {} -> {}", value, result);
    }
}
```

Output:
```
Processed 1 -> 2
Processed 2 -> 4
Processed 3 -> 6
Processed 4 -> 8
Processed 5 -> 10
```

Each processing step takes 100ms, but the runtime can handle other tasks during the waits.

## Using Stream Combinators

Once you have a stream, you can use powerful combinators:

### Example 1: Filter and Map

```rust
use tokio_stream;
use futures::StreamExt;

#[tokio::main]
async fn main() {
    let values = vec![1, 2, 3, 4, 5, 6, 7, 8, 9, 10];
    
    let mut stream = tokio_stream::iter(values)
        .filter(|x| futures::future::ready(x % 2 == 0))  // Only even
        .map(|x| x * x);                                   // Square them
    
    while let Some(value) = stream.next().await {
        println!("Value: {}", value);
    }
}
```

Output:
```
Value: 4
Value: 16
Value: 36
Value: 64
Value: 100
```

### Example 2: Take and Fold

```rust
use tokio_stream;
use futures::StreamExt;

#[tokio::main]
async fn main() {
    let values = vec![1, 2, 3, 4, 5, 6, 7, 8, 9, 10];
    
    let sum = tokio_stream::iter(values)
        .take(5)                      // Only first 5
        .fold(0, |acc, x| async move { acc + x })
        .await;
    
    println!("Sum of first 5: {}", sum);
}
```

Output: `Sum of first 5: 15`

### Example 3: Collect Back to Vec

```rust
use tokio_stream;
use futures::StreamExt;

#[tokio::main]
async fn main() {
    let values = vec![1, 2, 3, 4, 5];
    
    let doubled: Vec<i32> = tokio_stream::iter(values)
        .map(|x| x * 2)
        .collect()
        .await;
    
    println!("Doubled: {:?}", doubled);
}
```

Output: `Doubled: [2, 4, 6, 8, 10]`

## Real-World Example: Async HTTP Requests

```rust
use tokio_stream;
use futures::StreamExt;
use tokio::time::{sleep, Duration};

#[derive(Debug)]
struct User {
    id: i32,
    name: String,
}

async fn fetch_user(id: i32) -> User {
    // Simulate HTTP request
    sleep(Duration::from_millis(100)).await;
    User {
        id,
        name: format!("User{}", id),
    }
}

#[tokio::main]
async fn main() {
    let user_ids = vec![1, 2, 3, 4, 5];
    
    println!("Fetching users...");
    
    let mut stream = tokio_stream::iter(user_ids);
    
    while let Some(id) = stream.next().await {
        let user = fetch_user(id).await;
        println!("Fetched: {:?}", user);
    }
    
    println!("All users fetched!");
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
All users fetched!
```

## Advanced: Concurrent Processing with `buffer_unordered`

Process multiple items concurrently instead of sequentially:

```rust
use tokio_stream;
use futures::StreamExt;
use tokio::time::{sleep, Duration};

async fn fetch_user(id: i32) -> User {
    sleep(Duration::from_millis(100)).await;
    User {
        id,
        name: format!("User{}", id),
    }
}

#[tokio::main]
async fn main() {
    let user_ids = vec![1, 2, 3, 4, 5];
    
    let mut stream = tokio_stream::iter(user_ids)
        .map(|id| fetch_user(id))
        .buffer_unordered(3);  // Process 3 at a time
    
    while let Some(user) = stream.next().await {
        println!("Fetched: {:?}", user);
    }
}
```

This processes 3 users concurrently, potentially reducing total time from 500ms to ~200ms.

## Comparison: Iterator vs Stream

### Synchronous Iterator

```rust
let values = vec![1, 2, 3, 4, 5];

for value in values {
    // Blocking - must complete before next iteration
    expensive_operation(value);
}
```

**Characteristics:**
- Blocking execution
- Sequential only
- No concurrency
- Simple and fast for CPU-bound work

### Async Stream

```rust
let values = vec![1, 2, 3, 4, 5];
let mut stream = tokio_stream::iter(values);

while let Some(value) = stream.next().await {
    // Non-blocking - yields to runtime
    async_expensive_operation(value).await;
}
```

**Characteristics:**
- Non-blocking execution
- Can be concurrent
- Runtime can handle other tasks
- Better for I/O-bound work

## When to Use `tokio_stream::iter`

### Good Use Cases ✅

1. **Integrating sync data into async pipelines**
   ```rust
   let config_values = load_config();  // Sync
   let stream = tokio_stream::iter(config_values);
   // Now can use with other async streams
   ```

2. **Using stream combinators on collections**
   ```rust
   let processed = tokio_stream::iter(data)
       .filter(|x| ...)
       .map(|x| ...)
       .collect()
       .await;
   ```

3. **Async processing of collection items**
   ```rust
   tokio_stream::iter(urls)
       .for_each(|url| async move {
           fetch(url).await;
       })
       .await;
   ```

4. **Testing async stream consumers**
   ```rust
   let test_data = vec![1, 2, 3];
   let stream = tokio_stream::iter(test_data);
   test_stream_processor(stream).await;
   ```

### When NOT to Use ❌

1. **Simple CPU-bound iteration**
   ```rust
   // ❌ Unnecessary overhead
   let mut stream = tokio_stream::iter(vec![1, 2, 3]);
   while let Some(x) = stream.next().await {
       println!("{}", x);  // No async needed
   }
   
   // ✅ Just use regular iteration
   for x in vec![1, 2, 3] {
       println!("{}", x);
   }
   ```

2. **No async operations in processing**
   - If you're not doing any async work, regular iterators are faster

3. **Hot loops requiring maximum performance**
   - The async overhead (even minimal) may matter

## Key Differences from Regular Iteration

| Aspect | Regular Iterator | tokio_stream::iter |
|--------|------------------|-------------------|
| **Execution** | Synchronous | Asynchronous |
| **Blocking** | Blocks thread | Yields to runtime |
| **Performance** | Faster (no overhead) | Small overhead |
| **Concurrency** | None | Possible |
| **Combinators** | Iterator methods | Stream methods |
| **Use with async** | Cannot mix | Can mix freely |

## Best Practices

### 1. Only Convert When Needed

```rust
// ❌ Unnecessary conversion
let stream = tokio_stream::iter(vec![1, 2, 3]);
let sum = stream.fold(0, |a, b| async move { a + b }).await;

// ✅ Use regular iterator
let sum: i32 = vec![1, 2, 3].iter().sum();
```

### 2. Consider Cloning for Shared Access

```rust
let values = vec![1, 2, 3, 4, 5];

// Use the values as stream
let stream = tokio_stream::iter(values.clone());
process_stream(stream).await;

// Still have access to original
println!("Original: {:?}", values);
```

### 3. Use References for Large Collections

```rust
let large_data = vec![/* millions of items */];

// ✅ Avoid cloning large data
let stream = tokio_stream::iter(&large_data);
```

### 4. Combine with Other Streams

```rust
let static_data = tokio_stream::iter(vec![1, 2, 3]);
let channel_stream = ReceiverStream::new(rx);

let combined = static_data.chain(channel_stream);
```

## Summary

`tokio_stream::iter` bridges synchronous and asynchronous worlds by:

1. **Converting iterators to streams**: Takes any `IntoIterator` and creates a `Stream`
2. **Enabling async processing**: Allows async operations on each item
3. **Providing stream combinators**: Access to `filter`, `map`, `fold`, etc.
4. **Maintaining consistency**: Uniform interface with other async sources
5. **Supporting composition**: Can be combined with other streams

### When to Use

- Integrating sync collections into async code
- Using stream combinators on regular data
- Processing collection items with async operations
- Testing async stream consumers

### Pattern

```rust
let collection = vec![/* data */];
let mut stream = tokio_stream::iter(collection);

while let Some(item) = stream.next().await {
    // Async processing
}
```

This simple pattern enables powerful async processing of synchronous data, making it easy to mix sync and async code in Rust applications.