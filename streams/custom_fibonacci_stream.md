# Implementing a Custom Fibonacci Stream

## Overview

This code demonstrates how to implement a custom stream from scratch by implementing the **`Stream` trait**. Unlike using built-in stream constructors like `tokio_stream::iter()`, this shows how to create your own stream type with custom logic - in this case, an infinite stream that generates Fibonacci numbers on demand.

## Complete Code

```rust
use futures::stream::Stream;
use std::pin::Pin;
use std::task::{Context, Poll};

struct Fibonacci {
    curr: u64,
    next: u64,
}

impl Fibonacci {
    fn new() -> Self {
        Fibonacci { curr: 0, next: 1 }
    }
}

impl Stream for Fibonacci {
    type Item = u64;
    
    fn poll_next(mut self: Pin<&mut Self>, _cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        let current = self.curr;
        let next = self.next;
        
        self.curr = next;
        self.next = current + next;
        
        Poll::Ready(Some(current))
    }
}

use futures::StreamExt;

#[tokio::main]
async fn main() {
    let fib = Fibonacci::new();
    
    let first_10: Vec<u64> = fib.take(10).collect().await;
    println!("First 10 Fibonacci numbers: {:?}", first_10);
}
```

## Cargo.toml

```toml
[package]
name = "custom-fibonacci-stream"
version = "0.1.0"
edition = "2021"

[dependencies]
tokio = { version = "1", features = ["full"] }
futures = "0.3"
```

## Expected Output

```
First 10 Fibonacci numbers: [0, 1, 1, 2, 3, 5, 8, 13, 21, 34]
```

## What is the Stream Trait?

The `Stream` trait is the async equivalent of the `Iterator` trait. It represents a sequence of values that are produced asynchronously over time.

### Stream Trait Definition

```rust
pub trait Stream {
    type Item;
    
    fn poll_next(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>
    ) -> Poll<Option<Self::Item>>;
}
```

**Key components:**
- **`Item`**: The type of values produced by the stream
- **`poll_next`**: Method called to get the next value
- **Returns `Poll<Option<Item>>`**:
  - `Poll::Ready(Some(value))` - A value is ready
  - `Poll::Ready(None)` - Stream has ended
  - `Poll::Pending` - Not ready yet, will notify when ready

## How This Code Works

### Step 1: Define the Stream Structure

```rust
struct Fibonacci {
    curr: u64,
    next: u64,
}
```

**State management:**
- `curr`: The current Fibonacci number to be yielded
- `next`: The next Fibonacci number in the sequence

This struct holds the state needed to generate the Fibonacci sequence.

### Step 2: Constructor

```rust
impl Fibonacci {
    fn new() -> Self {
        Fibonacci { curr: 0, next: 1 }
    }
}
```

**Initial state:**
- `curr = 0` (first Fibonacci number)
- `next = 1` (second Fibonacci number)

This sets up the beginning of the Fibonacci sequence: 0, 1, 1, 2, 3, 5, 8, ...

### Step 3: Implement the Stream Trait

```rust
impl Stream for Fibonacci {
    type Item = u64;
    
    fn poll_next(mut self: Pin<&mut Self>, _cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        let current = self.curr;
        let next = self.next;
        
        self.curr = next;
        self.next = current + next;
        
        Poll::Ready(Some(current))
    }
}
```

**What happens in `poll_next`:**

1. **Save current state**: 
   ```rust
   let current = self.curr;  // Value to return
   let next = self.next;     // Next value in sequence
   ```

2. **Update state for next call**:
   ```rust
   self.curr = next;           // Move to next number
   self.next = current + next; // Calculate following number
   ```

3. **Return the current value**:
   ```rust
   Poll::Ready(Some(current))
   ```

**Important:** This stream is **infinite** - it never returns `Poll::Ready(None)`, so it will keep generating Fibonacci numbers forever (until integer overflow or stream is dropped).

### Step 4: Use the Stream

```rust
#[tokio::main]
async fn main() {
    let fib = Fibonacci::new();
    
    let first_10: Vec<u64> = fib.take(10).collect().await;
    println!("First 10 Fibonacci numbers: {:?}", first_10);
}
```

**Execution:**
1. Create a new Fibonacci stream
2. Use `.take(10)` to limit to first 10 values
3. Collect into a `Vec<u64>`
4. Print the results

## Visual Execution Flow

```
Initial State:
┌──────────────┐
│ curr: 0      │
│ next: 1      │
└──────────────┘

Poll 1:
  Save: current = 0, next = 1
  Update: curr = 1, next = 0 + 1 = 1
  Return: Some(0)
  State: curr=1, next=1

Poll 2:
  Save: current = 1, next = 1
  Update: curr = 1, next = 1 + 1 = 2
  Return: Some(1)
  State: curr=1, next=2

Poll 3:
  Save: current = 1, next = 2
  Update: curr = 2, next = 1 + 2 = 3
  Return: Some(1)
  State: curr=2, next=3

Poll 4:
  Save: current = 2, next = 3
  Update: curr = 3, next = 2 + 3 = 5
  Return: Some(2)
  State: curr=3, next=5

Poll 5:
  Save: current = 3, next = 5
  Update: curr = 5, next = 3 + 5 = 8
  Return: Some(3)
  State: curr=5, next=8

... continues infinitely
```

## Detailed Execution Trace

| Call | Before State | Current | Next | After State | Returned |
|------|-------------|---------|------|-------------|----------|
| 1 | curr=0, next=1 | 0 | 1 | curr=1, next=1 | 0 |
| 2 | curr=1, next=1 | 1 | 1 | curr=1, next=2 | 1 |
| 3 | curr=1, next=2 | 1 | 2 | curr=2, next=3 | 1 |
| 4 | curr=2, next=3 | 2 | 3 | curr=3, next=5 | 2 |
| 5 | curr=3, next=5 | 3 | 5 | curr=5, next=8 | 3 |
| 6 | curr=5, next=8 | 5 | 8 | curr=8, next=13 | 5 |
| 7 | curr=8, next=13 | 8 | 13 | curr=13, next=21 | 8 |
| 8 | curr=13, next=21 | 13 | 21 | curr=21, next=34 | 13 |
| 9 | curr=21, next=34 | 21 | 34 | curr=34, next=55 | 21 |
| 10 | curr=34, next=55 | 34 | 55 | curr=55, next=89 | 34 |

## Understanding the Stream Trait Components

### `type Item = u64`

Declares that this stream produces `u64` values:
```rust
type Item = u64;
```

This is equivalent to saying "this stream yields unsigned 64-bit integers."

### `Pin<&mut Self>`

The `Pin` ensures the struct's memory location doesn't move:
```rust
fn poll_next(mut self: Pin<&mut Self>, _cx: &mut Context<'_>)
```

**Why Pin?**
- Streams may contain self-referential data
- Async operations need stable memory addresses
- Pin guarantees the struct won't move in memory

### `Context<'_>`

The context provides a way to wake the task when new data is available:
```rust
_cx: &mut Context<'_>
```

**In this example:**
- We don't use it (hence `_cx`)
- Our stream always has data ready
- For async I/O streams, you'd use `cx.waker()` to schedule wakeups

### `Poll<Option<Self::Item>>`

The return type represents the stream's state:

```rust
Poll<Option<Self::Item>>
```

**Three possible returns:**

1. **`Poll::Ready(Some(value))`**: Value is ready
   ```rust
   Poll::Ready(Some(42))
   ```

2. **`Poll::Ready(None)`**: Stream has ended
   ```rust
   Poll::Ready(None)
   ```

3. **`Poll::Pending`**: Not ready, try again later
   ```rust
   Poll::Pending
   ```

## Finite vs Infinite Streams

### Infinite Stream (Our Example)

```rust
fn poll_next(mut self: Pin<&mut Self>, _cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
    // Always returns Some - never ends
    Poll::Ready(Some(self.compute_next()))
}
```

**Characteristics:**
- Never returns `None`
- Generates values forever
- Must be limited with `.take()` or similar

### Finite Stream Example

```rust
struct CountDown {
    remaining: u32,
}

impl Stream for CountDown {
    type Item = u32;
    
    fn poll_next(mut self: Pin<&mut Self>, _cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        if self.remaining == 0 {
            Poll::Ready(None)  // Stream ends
        } else {
            let current = self.remaining;
            self.remaining -= 1;
            Poll::Ready(Some(current))
        }
    }
}
```

## Practical Examples

### Example 1: Range Stream

```rust
use futures::stream::Stream;
use futures::StreamExt;
use std::pin::Pin;
use std::task::{Context, Poll};

struct Range {
    current: i32,
    end: i32,
}

impl Range {
    fn new(start: i32, end: i32) -> Self {
        Range { current: start, end }
    }
}

impl Stream for Range {
    type Item = i32;
    
    fn poll_next(mut self: Pin<&mut Self>, _cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        if self.current < self.end {
            let current = self.current;
            self.current += 1;
            Poll::Ready(Some(current))
        } else {
            Poll::Ready(None)
        }
    }
}

#[tokio::main]
async fn main() {
    let range = Range::new(1, 6);
    let values: Vec<i32> = range.collect().await;
    println!("Range: {:?}", values);
}
```

Output: `Range: [1, 2, 3, 4, 5]`

### Example 2: Doubling Stream

```rust
use futures::stream::Stream;
use futures::StreamExt;
use std::pin::Pin;
use std::task::{Context, Poll};

struct Doubler {
    current: u32,
    max: u32,
}

impl Doubler {
    fn new(max: u32) -> Self {
        Doubler { current: 1, max }
    }
}

impl Stream for Doubler {
    type Item = u32;
    
    fn poll_next(mut self: Pin<&mut Self>, _cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        if self.current <= self.max {
            let current = self.current;
            self.current *= 2;
            Poll::Ready(Some(current))
        } else {
            Poll::Ready(None)
        }
    }
}

#[tokio::main]
async fn main() {
    let doubler = Doubler::new(64);
    let values: Vec<u32> = doubler.collect().await;
    println!("Powers of 2: {:?}", values);
}
```

Output: `Powers of 2: [1, 2, 4, 8, 16, 32, 64]`

### Example 3: Prime Number Stream

```rust
use futures::stream::Stream;
use futures::StreamExt;
use std::pin::Pin;
use std::task::{Context, Poll};

struct Primes {
    current: u64,
}

impl Primes {
    fn new() -> Self {
        Primes { current: 2 }
    }
    
    fn is_prime(n: u64) -> bool {
        if n < 2 {
            return false;
        }
        for i in 2..=((n as f64).sqrt() as u64) {
            if n % i == 0 {
                return false;
            }
        }
        true
    }
}

impl Stream for Primes {
    type Item = u64;
    
    fn poll_next(mut self: Pin<&mut Self>, _cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        while !Self::is_prime(self.current) {
            self.current += 1;
        }
        
        let prime = self.current;
        self.current += 1;
        Poll::Ready(Some(prime))
    }
}

#[tokio::main]
async fn main() {
    let primes = Primes::new();
    let first_10: Vec<u64> = primes.take(10).collect().await;
    println!("First 10 primes: {:?}", first_10);
}
```

Output: `First 10 primes: [2, 3, 5, 7, 11, 13, 17, 19, 23, 29]`

### Example 4: Stateful Stream with External Data

```rust
use futures::stream::Stream;
use futures::StreamExt;
use std::pin::Pin;
use std::task::{Context, Poll};

struct Cycle<T: Clone> {
    items: Vec<T>,
    index: usize,
}

impl<T: Clone> Cycle<T> {
    fn new(items: Vec<T>) -> Self {
        Cycle { items, index: 0 }
    }
}

impl<T: Clone> Stream for Cycle<T> {
    type Item = T;
    
    fn poll_next(mut self: Pin<&mut Self>, _cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        if self.items.is_empty() {
            return Poll::Ready(None);
        }
        
        let item = self.items[self.index].clone();
        self.index = (self.index + 1) % self.items.len();
        Poll::Ready(Some(item))
    }
}

#[tokio::main]
async fn main() {
    let cycle = Cycle::new(vec!["A", "B", "C"]);
    let values: Vec<&str> = cycle.take(10).collect().await;
    println!("Cycled: {:?}", values);
}
```

Output: `Cycled: ["A", "B", "C", "A", "B", "C", "A", "B", "C", "A"]`

## Using Stream Combinators

Once you implement the `Stream` trait, you get all the combinator methods for free:

```rust
use futures::StreamExt;

#[tokio::main]
async fn main() {
    let fib = Fibonacci::new();
    
    // Take first 20
    let first_20: Vec<u64> = fib.take(20).collect().await;
    
    // Filter evens
    let fib = Fibonacci::new();
    let evens: Vec<u64> = fib
        .take(20)
        .filter(|x| futures::future::ready(x % 2 == 0))
        .collect()
        .await;
    
    // Map to strings
    let fib = Fibonacci::new();
    let strings: Vec<String> = fib
        .take(10)
        .map(|x| format!("Fib: {}", x))
        .collect()
        .await;
    
    // Sum first 10
    let fib = Fibonacci::new();
    let sum: u64 = fib
        .take(10)
        .fold(0, |acc, x| async move { acc + x })
        .await;
    
    println!("Sum of first 10: {}", sum);
}
```

## Comparison: Custom Stream vs Built-in Constructors

### Custom Stream (Manual Implementation)

```rust
struct Fibonacci { /* ... */ }

impl Stream for Fibonacci {
    fn poll_next(/* ... */) -> Poll<Option<u64>> {
        // Custom logic
    }
}
```

**Pros:**
- Full control over logic
- Can maintain complex state
- Efficient for custom patterns

**Cons:**
- More code to write
- Need to understand `Pin` and `Poll`
- Manual state management

### Built-in Constructor

```rust
let stream = tokio_stream::iter(vec![0, 1, 1, 2, 3, 5, 8]);
```

**Pros:**
- Simple and concise
- No boilerplate
- Works for common cases

**Cons:**
- Limited to provided constructors
- May not fit custom needs
- Less control over behavior

## When to Implement Custom Streams

### Good Use Cases ✅

1. **Mathematical sequences**: Fibonacci, primes, factorials
2. **Stateful generation**: Sequences with complex state
3. **External data sources**: Reading from files, sockets
4. **Custom protocols**: Implementing streaming protocols
5. **Infinite streams**: Generating endless sequences
6. **Performance-critical**: Optimized generation logic

### When to Use Built-in Constructors ✅

1. **Simple cases**: Known data, simple iteration
2. **One-time use**: No reusable logic needed
3. **Rapid prototyping**: Quick solutions
4. **Standard patterns**: Channel wrapping, range iteration

## Advanced: Async Stream with Poll::Pending

For streams that need to wait for async operations:

```rust
use futures::stream::Stream;
use std::pin::Pin;
use std::task::{Context, Poll, Waker};
use std::sync::{Arc, Mutex};

struct DelayedStream {
    values: Vec<i32>,
    index: usize,
    waker: Arc<Mutex<Option<Waker>>>,
}

impl Stream for DelayedStream {
    type Item = i32;
    
    fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        if self.index < self.values.len() {
            // Simulate async waiting
            if self.index % 2 == 0 {
                // Sometimes return Pending
                *self.waker.lock().unwrap() = Some(cx.waker().clone());
                // In real code, spawn task to wake later
                Poll::Pending
            } else {
                let value = self.values[self.index];
                self.index += 1;
                Poll::Ready(Some(value))
            }
        } else {
            Poll::Ready(None)
        }
    }
}
```

## Best Practices

### 1. Initialize State Properly

```rust
impl Fibonacci {
    fn new() -> Self {
        Fibonacci { curr: 0, next: 1 }  // Correct initial state
    }
}
```

### 2. Update State Correctly

```rust
// ✅ Good: Calculate before updating
let current = self.curr;
let next = self.next;
self.curr = next;
self.next = current + next;

// ❌ Bad: Wrong order
self.next = self.curr + self.next;  // Uses updated curr!
self.curr = self.next;
```

### 3. Return Appropriate Poll Values

```rust
// For infinite streams
Poll::Ready(Some(value))

// For finite streams
if self.has_more() {
    Poll::Ready(Some(value))
} else {
    Poll::Ready(None)
}
```

### 4. Use `_cx` if Context Not Needed

```rust
fn poll_next(mut self: Pin<&mut Self>, _cx: &mut Context<'_>)
//                                       ^ underscore prefix
```

## Summary

Implementing a custom stream involves:

1. **Define a struct**: Hold the state needed for generation
2. **Implement `Stream` trait**: Define how values are produced
3. **Specify `Item` type**: What type of values the stream yields
4. **Implement `poll_next`**: Core logic for generating values
5. **Return appropriate `Poll`**: `Ready(Some)`, `Ready(None)`, or `Pending`

### Basic Pattern

```rust
struct MyStream {
    // State fields
}

impl Stream for MyStream {
    type Item = YourType;
    
    fn poll_next(mut self: Pin<&mut Self>, _cx: &mut Context<'_>) 
        -> Poll<Option<Self::Item>> 
    {
        // Generate next value
        // Update state
        // Return Poll::Ready(Some(value)) or Poll::Ready(None)
    }
}
```

### When to Implement Custom Streams

- Need custom value generation logic
- Want to maintain complex state
- Implementing infinite sequences
- Building reusable stream types
- Need fine-grained control over stream behavior

Custom streams provide the foundation for building sophisticated async data pipelines in Rust, enabling you to create exactly the streaming behavior your application needs.