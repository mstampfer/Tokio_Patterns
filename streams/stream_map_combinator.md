# Transforming Stream Values with the `map` Combinator

## Overview

This code demonstrates how to use the **`map` combinator** to transform values in a stream. Stream combinators allow you to build data processing pipelines by chaining operations together, similar to iterator combinators but in the async world.

## Complete Code

```rust
use tokio_stream;
use futures::StreamExt;

#[tokio::main]
async fn main() {
    let stream = tokio_stream::iter(1..=5)
        .map(|x| x * 2);
    
    let results: Vec<i32> = stream.collect().await;
    println!("Results: {:?}", results);
}
```

### Cargo.toml

```toml
[package]
name = "stream-map-example"
version = "0.1.0"
edition = "2021"

[dependencies]
tokio = { version = "1", features = ["full"] }
tokio-stream = "0.1"
futures = "0.3"
```

### Expected Output

```
Results: [2, 4, 6, 8, 10]
```

## What is the `map` Combinator?

The `map` combinator transforms each value in a stream by applying a function to it. It's the stream equivalent of the iterator `map` method.

### Function Signature

```rust
fn map<T, F>(self, f: F) -> Map<Self, F>
where
    F: FnMut(Self::Item) -> T,
```

**Key points:**
- Takes a closure that transforms each item
- Returns a new stream with transformed values
- Does not consume the original stream immediately
- Lazy evaluation - transformations happen as values are consumed

## How This Code Works

### Step 1: Create a Stream from Range

```rust
let stream = tokio_stream::iter(1..=5)
```

Creates a stream from the range `1..=5`, which produces values: `1, 2, 3, 4, 5`

**Stream state:**
```
Stream: [1, 2, 3, 4, 5]
```

### Step 2: Apply `map` Transformation

```rust
    .map(|x| x * 2);
```

The `map` combinator:
1. Takes each value `x` from the stream
2. Applies the closure `|x| x * 2` to it
3. Produces a new stream with doubled values

**Transformation:**
```
Input:  [1,    2,    3,    4,    5]
         ↓     ↓     ↓     ↓     ↓
        ×2    ×2    ×2    ×2    ×2
         ↓     ↓     ↓     ↓     ↓
Output: [2,    4,    6,    8,   10]
```

**Important:** The transformation doesn't happen yet - it's lazy. The closure will only be called when values are pulled from the stream.

### Step 3: Collect Results

```rust
let results: Vec<i32> = stream.collect().await;
```

**What happens:**
1. `collect()` consumes the stream, pulling all values
2. As each value is pulled, the `map` closure executes
3. Transformed values are collected into a `Vec<i32>`
4. `.await` is required because `collect()` is async

**Execution flow:**
```
1. Pull first value → 1 → map(1) → 2 → add to vec
2. Pull second value → 2 → map(2) → 4 → add to vec
3. Pull third value → 3 → map(3) → 6 → add to vec
4. Pull fourth value → 4 → map(4) → 8 → add to vec
5. Pull fifth value → 5 → map(5) → 10 → add to vec
6. Stream exhausted → return vec
```

### Step 4: Print Results

```rust
println!("Results: {:?}", results);
```

Prints: `Results: [2, 4, 6, 8, 10]`

## Visual Data Flow

```
┌─────────────────┐
│ tokio_stream    │
│   ::iter(1..=5) │
└────────┬────────┘
         │ Stream<Item = i32>
         │ Values: 1, 2, 3, 4, 5
         ▼
┌─────────────────┐
│  .map(|x| x*2)  │  ← Transformation applied
└────────┬────────┘
         │ Stream<Item = i32>
         │ Values: 2, 4, 6, 8, 10
         ▼
┌─────────────────┐
│ .collect().await│  ← Consume & collect
└────────┬────────┘
         │
         ▼
   Vec<i32>
   [2, 4, 6, 8, 10]
```

## Lazy Evaluation

The `map` transformation is **lazy** - it doesn't execute until values are consumed:

```rust
// Map is defined but not executed yet
let stream = tokio_stream::iter(1..=5)
    .map(|x| {
        println!("Mapping {}", x);  // Won't print yet!
        x * 2
    });

println!("Stream created, but map hasn't run");

// Now map executes as we consume values
let results: Vec<i32> = stream.collect().await;
```

Output:
```
Stream created, but map hasn't run
Mapping 1
Mapping 2
Mapping 3
Mapping 4
Mapping 5
```

## Comparison: Iterator vs Stream `map`

### Synchronous Iterator

```rust
let values = vec![1, 2, 3, 4, 5];
let results: Vec<i32> = values
    .iter()
    .map(|x| x * 2)
    .collect();

println!("Results: {:?}", results);
```

**Characteristics:**
- Synchronous execution
- No `.await` needed
- Immediate (blocking) collection

### Async Stream

```rust
let stream = tokio_stream::iter(1..=5)
    .map(|x| x * 2);

let results: Vec<i32> = stream.collect().await;
println!("Results: {:?}", results);
```

**Characteristics:**
- Asynchronous execution
- Requires `.await`
- Non-blocking collection

## Multiple `map` Operations

You can chain multiple `map` operations:

```rust
use tokio_stream;
use futures::StreamExt;

#[tokio::main]
async fn main() {
    let stream = tokio_stream::iter(1..=5)
        .map(|x| x * 2)      // Double: 2, 4, 6, 8, 10
        .map(|x| x + 1)      // Add 1: 3, 5, 7, 9, 11
        .map(|x| x * x);     // Square: 9, 25, 49, 81, 121
    
    let results: Vec<i32> = stream.collect().await;
    println!("Results: {:?}", results);
}
```

Output: `Results: [9, 25, 49, 81, 121]`

**Transformation pipeline:**
```
1 → ×2 → 2 → +1 → 3 → ×3 → 9
2 → ×2 → 4 → +1 → 5 → ×5 → 25
3 → ×2 → 6 → +1 → 7 → ×7 → 49
4 → ×2 → 8 → +1 → 9 → ×9 → 81
5 → ×2 → 10 → +1 → 11 → ×11 → 121
```

## Async Transformations with `then`

For **async** transformations, use `then` instead of `map`:

```rust
use tokio_stream;
use futures::StreamExt;
use tokio::time::{sleep, Duration};

async fn fetch_data(id: i32) -> String {
    sleep(Duration::from_millis(100)).await;
    format!("Data for ID {}", id)
}

#[tokio::main]
async fn main() {
    let stream = tokio_stream::iter(1..=5)
        .then(|id| async move {
            fetch_data(id).await
        });
    
    let results: Vec<String> = stream.collect().await;
    println!("Results: {:?}", results);
}
```

Output:
```
Results: ["Data for ID 1", "Data for ID 2", "Data for ID 3", "Data for ID 4", "Data for ID 5"]
```

**Key difference:**
- `map`: Synchronous closure `|x| x * 2`
- `then`: Async closure returning a future `|x| async move { ... }`

## Common Stream Combinators

### 1. `map` - Transform Values

```rust
let stream = tokio_stream::iter(1..=5)
    .map(|x| x * 2);

let results: Vec<i32> = stream.collect().await;
// [2, 4, 6, 8, 10]
```

### 2. `filter` - Keep Some Values

```rust
let stream = tokio_stream::iter(1..=10)
    .filter(|x| futures::future::ready(x % 2 == 0));

let results: Vec<i32> = stream.collect().await;
// [2, 4, 6, 8, 10]
```

### 3. `filter_map` - Filter and Transform

```rust
let stream = tokio_stream::iter(1..=10)
    .filter_map(|x| {
        if x % 2 == 0 {
            futures::future::ready(Some(x * 2))
        } else {
            futures::future::ready(None)
        }
    });

let results: Vec<i32> = stream.collect().await;
// [4, 8, 12, 16, 20]
```

### 4. `take` - Limit Number of Items

```rust
let stream = tokio_stream::iter(1..=100)
    .map(|x| x * 2)
    .take(5);

let results: Vec<i32> = stream.collect().await;
// [2, 4, 6, 8, 10]
```

### 5. `skip` - Skip First N Items

```rust
let stream = tokio_stream::iter(1..=10)
    .map(|x| x * 2)
    .skip(3);

let results: Vec<i32> = stream.collect().await;
// [8, 10, 12, 14, 16, 18, 20]
```

### 6. `fold` - Reduce to Single Value

```rust
let sum = tokio_stream::iter(1..=5)
    .map(|x| x * 2)
    .fold(0, |acc, x| async move { acc + x })
    .await;

println!("Sum: {}", sum);
// Sum: 30
```

## Practical Examples

### Example 1: Data Processing Pipeline

```rust
use tokio_stream;
use futures::StreamExt;

#[derive(Debug)]
struct User {
    id: i32,
    name: String,
    age: i32,
}

#[tokio::main]
async fn main() {
    let user_ids = vec![1, 2, 3, 4, 5];
    
    let users: Vec<User> = tokio_stream::iter(user_ids)
        .map(|id| User {
            id,
            name: format!("User{}", id),
            age: 20 + id,
        })
        .collect()
        .await;
    
    println!("Users: {:?}", users);
}
```

Output:
```
Users: [
    User { id: 1, name: "User1", age: 21 },
    User { id: 2, name: "User2", age: 22 },
    User { id: 3, name: "User3", age: 23 },
    User { id: 4, name: "User4", age: 24 },
    User { id: 5, name: "User5", age: 25 }
]
```

### Example 2: Temperature Conversion

```rust
use tokio_stream;
use futures::StreamExt;

#[tokio::main]
async fn main() {
    let celsius_temps = vec![0, 10, 20, 30, 40];
    
    let fahrenheit_temps: Vec<f64> = tokio_stream::iter(celsius_temps)
        .map(|c| (c as f64 * 9.0 / 5.0) + 32.0)
        .collect()
        .await;
    
    println!("Fahrenheit: {:?}", fahrenheit_temps);
}
```

Output: `Fahrenheit: [32.0, 50.0, 68.0, 86.0, 104.0]`

### Example 3: String Processing

```rust
use tokio_stream;
use futures::StreamExt;

#[tokio::main]
async fn main() {
    let words = vec!["hello", "world", "rust", "async"];
    
    let processed: Vec<String> = tokio_stream::iter(words)
        .map(|s| s.to_uppercase())
        .map(|s| format!("{}!", s))
        .collect()
        .await;
    
    println!("Processed: {:?}", processed);
}
```

Output: `Processed: ["HELLO!", "WORLD!", "RUST!", "ASYNC!"]`

### Example 4: Complex Data Transformation

```rust
use tokio_stream;
use futures::StreamExt;

#[derive(Debug)]
struct Product {
    name: String,
    price: f64,
}

#[derive(Debug)]
struct PriceTag {
    product: String,
    original_price: f64,
    discounted_price: f64,
    savings: f64,
}

#[tokio::main]
async fn main() {
    let products = vec![
        Product { name: "Laptop".to_string(), price: 1000.0 },
        Product { name: "Mouse".to_string(), price: 50.0 },
        Product { name: "Keyboard".to_string(), price: 100.0 },
    ];
    
    let discount_rate = 0.20;  // 20% off
    
    let price_tags: Vec<PriceTag> = tokio_stream::iter(products)
        .map(|p| {
            let discounted = p.price * (1.0 - discount_rate);
            PriceTag {
                product: p.name,
                original_price: p.price,
                discounted_price: discounted,
                savings: p.price - discounted,
            }
        })
        .collect()
        .await;
    
    for tag in price_tags {
        println!("{}: ${:.2} (Save ${:.2})", 
                 tag.product, tag.discounted_price, tag.savings);
    }
}
```

Output:
```
Laptop: $800.00 (Save $200.00)
Mouse: $40.00 (Save $10.00)
Keyboard: $80.00 (Save $20.00)
```

## Combining `map` with Other Combinators

### Filter then Map

```rust
use tokio_stream;
use futures::StreamExt;

#[tokio::main]
async fn main() {
    let results: Vec<i32> = tokio_stream::iter(1..=10)
        .filter(|x| futures::future::ready(x % 2 == 0))  // Keep evens
        .map(|x| x * x)                                    // Square them
        .collect()
        .await;
    
    println!("Results: {:?}", results);
}
```

Output: `Results: [4, 16, 36, 64, 100]`

### Map then Take

```rust
use tokio_stream;
use futures::StreamExt;

#[tokio::main]
async fn main() {
    let results: Vec<i32> = tokio_stream::iter(1..=100)
        .map(|x| x * x)     // Square all
        .take(5)            // But only take first 5
        .collect()
        .await;
    
    println!("Results: {:?}", results);
}
```

Output: `Results: [1, 4, 9, 16, 25]`

### Map then Fold

```rust
use tokio_stream;
use futures::StreamExt;

#[tokio::main]
async fn main() {
    let sum: i32 = tokio_stream::iter(1..=5)
        .map(|x| x * 2)     // Double: 2, 4, 6, 8, 10
        .fold(0, |acc, x| async move { acc + x })
        .await;
    
    println!("Sum: {}", sum);
}
```

Output: `Sum: 30`

## Performance Considerations

### Efficient Chaining

```rust
// ✅ Good: Chained transformations are efficient
let stream = tokio_stream::iter(1..=1000)
    .map(|x| x * 2)
    .filter(|x| futures::future::ready(x % 3 == 0))
    .map(|x| x + 1);
// Single pass through data
```

### Avoid Unnecessary Collections

```rust
// ❌ Bad: Collecting intermediate results
let doubled: Vec<i32> = tokio_stream::iter(1..=100)
    .map(|x| x * 2)
    .collect()
    .await;

let results: Vec<i32> = tokio_stream::iter(doubled)
    .filter(|x| futures::future::ready(x % 3 == 0))
    .collect()
    .await;

// ✅ Good: Chain operations without intermediate collections
let results: Vec<i32> = tokio_stream::iter(1..=100)
    .map(|x| x * 2)
    .filter(|x| futures::future::ready(x % 3 == 0))
    .collect()
    .await;
```

## Common Patterns

### Pattern 1: Type Conversion

```rust
let strings: Vec<String> = tokio_stream::iter(1..=5)
    .map(|x| x.to_string())
    .collect()
    .await;
```

### Pattern 2: Error Handling

```rust
use tokio_stream;
use futures::StreamExt;

fn process(x: i32) -> Result<i32, String> {
    if x > 0 {
        Ok(x * 2)
    } else {
        Err("Negative number".to_string())
    }
}

#[tokio::main]
async fn main() {
    let results: Vec<Result<i32, String>> = tokio_stream::iter(vec![-1, 2, 3, -4, 5])
        .map(|x| process(x))
        .collect()
        .await;
    
    println!("Results: {:?}", results);
}
```

### Pattern 3: Nested Data Structures

```rust
let nested: Vec<Vec<i32>> = tokio_stream::iter(1..=3)
    .map(|x| vec![x, x * 2, x * 3])
    .collect()
    .await;

// Result: [[1, 2, 3], [2, 4, 6], [3, 6, 9]]
```

## Best Practices

### 1. Keep Closures Simple

```rust
// ✅ Good: Simple, clear transformation
.map(|x| x * 2)

// ❌ Bad: Complex logic in closure
.map(|x| {
    let result = x * 2;
    let adjusted = result + 10;
    let final_value = adjusted / 3;
    final_value
})
// Better: Extract to a function
```

### 2. Use Meaningful Names

```rust
// ✅ Good: Clear parameter names
.map(|temperature_celsius| temperature_celsius * 9.0 / 5.0 + 32.0)

// ❌ Okay but less clear
.map(|x| x * 9.0 / 5.0 + 32.0)
```

### 3. Type Annotations When Needed

```rust
// Sometimes needed for clarity
let results: Vec<i32> = stream
    .map(|x| x * 2)
    .collect()
    .await;
```

### 4. Chain Operations Logically

```rust
// ✅ Good: Logical flow
stream
    .filter(|x| futures::future::ready(*x > 0))  // 1. Filter
    .map(|x| x * 2)                               // 2. Transform
    .take(10)                                     // 3. Limit
```

## Summary

The `map` combinator is fundamental for stream processing:

1. **Transforms each value**: Applies a function to every stream item
2. **Lazy evaluation**: Only executes when values are consumed
3. **Chainable**: Can be combined with other combinators
4. **Type-safe**: Maintains type information through transformations
5. **Efficient**: Single pass through data when chained

### Basic Pattern

```rust
let stream = tokio_stream::iter(source_data)
    .map(|item| transform(item));

let results: Vec<T> = stream.collect().await;
```

### When to Use `map`

- **Simple transformations**: Converting types, scaling values
- **Data enrichment**: Adding computed fields
- **Format conversion**: Changing data structure
- **Pipeline building**: Part of a larger processing chain

The `map` combinator, along with other stream combinators like `filter`, `fold`, and `take`, enables powerful functional-style data processing in async Rust applications.