# Filtering Stream Values with the `filter` Combinator

## Overview

This code demonstrates how to use the **`filter` combinator** to selectively keep values in a stream based on a condition. The `filter` combinator is the async equivalent of the iterator `filter` method, allowing you to build data processing pipelines that only include values meeting specific criteria.

## Complete Code

```rust
use tokio_stream;
use futures::StreamExt;

#[tokio::main]
async fn main() {
    let stream = tokio_stream::iter(1..=10)
        .filter(|x| futures::future::ready(x % 2 == 0));
    
    let evens: Vec<i32> = stream.collect().await;
    println!("Even numbers: {:?}", evens);
}
```

## Cargo.toml

```toml
[package]
name = "stream-filter-example"
version = "0.1.0"
edition = "2021"

[dependencies]
tokio = { version = "1", features = ["full"] }
tokio-stream = "0.1"
futures = "0.3"
```

## Expected Output

```
Even numbers: [2, 4, 6, 8, 10]
```

## What is the `filter` Combinator?

The `filter` combinator creates a new stream that only yields values for which a predicate returns `true`. Values that don't match the condition are skipped.

### Function Signature

```rust
fn filter<F>(self, f: F) -> Filter<Self, F>
where
    F: FnMut(&Self::Item) -> impl Future<Output = bool>,
```

**Key characteristics:**
- Takes a closure that returns a `Future<Output = bool>`
- Returns a new filtered stream
- Lazy evaluation - filtering happens when values are pulled
- Values are tested in order

## How This Code Works

### Step 1: Create a Stream from Range

```rust
let stream = tokio_stream::iter(1..=10)
```

Creates a stream that yields values from 1 to 10 (inclusive):
```
Stream: [1, 2, 3, 4, 5, 6, 7, 8, 9, 10]
```

### Step 2: Apply Filter Predicate

```rust
    .filter(|x| futures::future::ready(x % 2 == 0));
```

**What happens:**

1. **`|x|`**: For each value `x` in the stream
2. **`x % 2 == 0`**: Test if `x` is even (remainder of division by 2 is 0)
3. **`futures::future::ready(...)`**: Wrap the boolean result in a `Future`
4. If `true`: Keep the value in the stream
5. If `false`: Skip the value

**Filtering process:**
```
Value   Test (x % 2 == 0)   Result
-----   -----------------   ------
1       1 % 2 == 0 → false  Skip
2       2 % 2 == 0 → true   Keep  ✓
3       3 % 2 == 0 → false  Skip
4       4 % 2 == 0 → true   Keep  ✓
5       5 % 2 == 0 → false  Skip
6       6 % 2 == 0 → true   Keep  ✓
7       7 % 2 == 0 → false  Skip
8       8 % 2 == 0 → true   Keep  ✓
9       9 % 2 == 0 → false  Skip
10      10 % 2 == 0 → true  Keep  ✓
```

**Filtered stream:**
```
Stream: [2, 4, 6, 8, 10]
```

### Step 3: Collect Results

```rust
let evens: Vec<i32> = stream.collect().await;
```

**What happens:**
1. `collect()` consumes the stream
2. As each value is pulled, the filter predicate is evaluated
3. Only values that pass the filter are added to the vector
4. `.await` is required because `collect()` is async

### Step 4: Print Results

```rust
println!("Even numbers: {:?}", evens);
```

Prints: `Even numbers: [2, 4, 6, 8, 10]`

## Visual Data Flow

```
Original Stream
┌──────────────────────────────────────┐
│ 1, 2, 3, 4, 5, 6, 7, 8, 9, 10        │
└──────────────┬───────────────────────┘
               │
               ▼
       Filter: x % 2 == 0
               │
    ┌──────────┼──────────┐
    │ Test each value:    │
    │ 1 → false (skip)    │
    │ 2 → true  (keep)    │
    │ 3 → false (skip)    │
    │ 4 → true  (keep)    │
    │ 5 → false (skip)    │
    │ 6 → true  (keep)    │
    │ 7 → false (skip)    │
    │ 8 → true  (keep)    │
    │ 9 → false (skip)    │
    │ 10 → true (keep)    │
    └──────────┬──────────┘
               │
               ▼
      Filtered Stream
┌──────────────────────┐
│ 2, 4, 6, 8, 10       │
└──────────────────────┘
               │
               ▼
          collect()
               │
               ▼
         Vec<i32>
    [2, 4, 6, 8, 10]
```

## Why `futures::future::ready()`?

### The Problem

Stream's `filter` expects a closure that returns a `Future<Output = bool>`, not just a `bool`:

```rust
// ❌ This won't compile!
.filter(|x| x % 2 == 0)
// Error: expected Future, found bool
```

### The Solution

Wrap the boolean in a `Future` using `futures::future::ready()`:

```rust
// ✅ This works!
.filter(|x| futures::future::ready(x % 2 == 0))
```

**What `futures::future::ready()` does:**
- Takes any value
- Returns a `Future` that immediately resolves to that value
- Allows synchronous predicates to work with async streams

### For Async Predicates

If your predicate needs to perform async operations:

```rust
.filter(|x| async move {
    // Can use .await here
    let result = check_database(x).await;
    result.is_valid
})
```

## Comparison: Iterator vs Stream Filter

### Synchronous Iterator

```rust
let evens: Vec<i32> = (1..=10)
    .filter(|x| x % 2 == 0)  // Direct boolean
    .collect();

println!("Even numbers: {:?}", evens);
```

**Characteristics:**
- Simple boolean predicate
- No async overhead
- Blocking execution

### Async Stream

```rust
let stream = tokio_stream::iter(1..=10)
    .filter(|x| futures::future::ready(x % 2 == 0));  // Future<bool>

let evens: Vec<i32> = stream.collect().await;
println!("Even numbers: {:?}", evens);
```

**Characteristics:**
- Predicate returns `Future<bool>`
- Async overhead (minimal for simple predicates)
- Non-blocking execution

## Common Filter Patterns

### Pattern 1: Simple Conditions

```rust
// Keep positive numbers
.filter(|x| futures::future::ready(*x > 0))

// Keep strings longer than 5 characters
.filter(|s| futures::future::ready(s.len() > 5))

// Keep Some values, skip None
.filter(|opt| futures::future::ready(opt.is_some()))
```

### Pattern 2: Range Checks

```rust
use tokio_stream;
use futures::StreamExt;

#[tokio::main]
async fn main() {
    let in_range: Vec<i32> = tokio_stream::iter(1..=20)
        .filter(|x| futures::future::ready(*x >= 5 && *x <= 15))
        .collect()
        .await;
    
    println!("In range [5, 15]: {:?}", in_range);
}
```

Output: `In range [5, 15]: [5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15]`

### Pattern 3: Multiple Conditions

```rust
use tokio_stream;
use futures::StreamExt;

#[tokio::main]
async fn main() {
    let results: Vec<i32> = tokio_stream::iter(1..=20)
        .filter(|x| futures::future::ready(
            x % 2 == 0 && x % 3 == 0  // Divisible by both 2 and 3
        ))
        .collect()
        .await;
    
    println!("Divisible by 6: {:?}", results);
}
```

Output: `Divisible by 6: [6, 12, 18]`

### Pattern 4: String Filtering

```rust
use tokio_stream;
use futures::StreamExt;

#[tokio::main]
async fn main() {
    let words = vec!["apple", "banana", "cherry", "date", "elderberry"];
    
    let long_words: Vec<&str> = tokio_stream::iter(words)
        .filter(|word| futures::future::ready(word.len() > 5))
        .collect()
        .await;
    
    println!("Long words: {:?}", long_words);
}
```

Output: `Long words: ["banana", "cherry", "elderberry"]`

### Pattern 5: Filtering Structs

```rust
use tokio_stream;
use futures::StreamExt;

#[derive(Debug)]
struct Person {
    name: String,
    age: i32,
}

#[tokio::main]
async fn main() {
    let people = vec![
        Person { name: "Alice".to_string(), age: 25 },
        Person { name: "Bob".to_string(), age: 17 },
        Person { name: "Charlie".to_string(), age: 30 },
        Person { name: "Diana".to_string(), age: 16 },
    ];
    
    let adults: Vec<Person> = tokio_stream::iter(people)
        .filter(|person| futures::future::ready(person.age >= 18))
        .collect()
        .await;
    
    println!("Adults: {:?}", adults);
}
```

Output:
```
Adults: [
    Person { name: "Alice", age: 25 },
    Person { name: "Charlie", age: 30 }
]
```

## Async Filter Predicates

When your filter condition requires async operations:

```rust
use tokio_stream;
use futures::StreamExt;
use tokio::time::{sleep, Duration};

async fn is_valid(x: i32) -> bool {
    // Simulate async validation (e.g., database check, API call)
    sleep(Duration::from_millis(10)).await;
    x % 3 == 0
}

#[tokio::main]
async fn main() {
    let results: Vec<i32> = tokio_stream::iter(1..=10)
        .filter(|x| async move {
            is_valid(*x).await
        })
        .collect()
        .await;
    
    println!("Valid numbers: {:?}", results);
}
```

Output: `Valid numbers: [3, 6, 9]`

**Note:** Each filter check executes sequentially. If you need concurrent filtering, use `filter_map` with `buffer_unordered`.

## Combining `filter` with Other Combinators

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
    
    println!("Squared evens: {:?}", results);
}
```

Output: `Squared evens: [4, 16, 36, 64, 100]`

### Map then Filter

```rust
use tokio_stream;
use futures::StreamExt;

#[tokio::main]
async fn main() {
    let results: Vec<i32> = tokio_stream::iter(1..=10)
        .map(|x| x * 2)                                    // Double all
        .filter(|x| futures::future::ready(*x > 10))       // Keep > 10
        .collect()
        .await;
    
    println!("Doubled > 10: {:?}", results);
}
```

Output: `Doubled > 10: [12, 14, 16, 18, 20]`

### Filter, Map, Take

```rust
use tokio_stream;
use futures::StreamExt;

#[tokio::main]
async fn main() {
    let results: Vec<i32> = tokio_stream::iter(1..=100)
        .filter(|x| futures::future::ready(x % 2 == 0))   // Keep evens
        .map(|x| x * x)                                    // Square them
        .take(5)                                           // Only first 5
        .collect()
        .await;
    
    println!("First 5 squared evens: {:?}", results);
}
```

Output: `First 5 squared evens: [4, 16, 36, 64, 100]`

## Practical Examples

### Example 1: Filtering Validation Results

```rust
use tokio_stream;
use futures::StreamExt;

#[derive(Debug)]
struct ValidationResult {
    id: i32,
    valid: bool,
    message: String,
}

#[tokio::main]
async fn main() {
    let results = vec![
        ValidationResult { id: 1, valid: true, message: "OK".to_string() },
        ValidationResult { id: 2, valid: false, message: "Invalid format".to_string() },
        ValidationResult { id: 3, valid: true, message: "OK".to_string() },
        ValidationResult { id: 4, valid: false, message: "Missing field".to_string() },
    ];
    
    let valid_only: Vec<ValidationResult> = tokio_stream::iter(results)
        .filter(|r| futures::future::ready(r.valid))
        .collect()
        .await;
    
    println!("Valid results: {:?}", valid_only);
}
```

Output:
```
Valid results: [
    ValidationResult { id: 1, valid: true, message: "OK" },
    ValidationResult { id: 3, valid: true, message: "OK" }
]
```

### Example 2: Filtering API Responses

```rust
use tokio_stream;
use futures::StreamExt;
use tokio::time::{sleep, Duration};

#[derive(Debug)]
struct ApiResponse {
    status_code: u16,
    data: String,
}

async fn fetch_data(id: i32) -> ApiResponse {
    sleep(Duration::from_millis(10)).await;
    if id % 3 == 0 {
        ApiResponse {
            status_code: 200,
            data: format!("Data for ID {}", id),
        }
    } else {
        ApiResponse {
            status_code: 404,
            data: "Not found".to_string(),
        }
    }
}

#[tokio::main]
async fn main() {
    let ids = vec![1, 2, 3, 4, 5, 6, 7, 8, 9];
    
    let successful: Vec<ApiResponse> = tokio_stream::iter(ids)
        .then(|id| fetch_data(id))
        .filter(|response| futures::future::ready(response.status_code == 200))
        .collect()
        .await;
    
    println!("Successful responses: {:?}", successful);
}
```

Output:
```
Successful responses: [
    ApiResponse { status_code: 200, data: "Data for ID 3" },
    ApiResponse { status_code: 200, data: "Data for ID 6" },
    ApiResponse { status_code: 200, data: "Data for ID 9" }
]
```

### Example 3: Filtering File Paths

```rust
use tokio_stream;
use futures::StreamExt;

#[tokio::main]
async fn main() {
    let paths = vec![
        "document.txt",
        "image.png",
        "script.rs",
        "data.json",
        "code.rs",
        "photo.jpg",
    ];
    
    let rust_files: Vec<&str> = tokio_stream::iter(paths)
        .filter(|path| futures::future::ready(path.ends_with(".rs")))
        .collect()
        .await;
    
    println!("Rust files: {:?}", rust_files);
}
```

Output: `Rust files: ["script.rs", "code.rs"]`

### Example 4: Filtering by Price Range

```rust
use tokio_stream;
use futures::StreamExt;

#[derive(Debug)]
struct Product {
    name: String,
    price: f64,
}

#[tokio::main]
async fn main() {
    let products = vec![
        Product { name: "Mouse".to_string(), price: 25.99 },
        Product { name: "Keyboard".to_string(), price: 89.99 },
        Product { name: "Monitor".to_string(), price: 299.99 },
        Product { name: "Cable".to_string(), price: 9.99 },
        Product { name: "Laptop".to_string(), price: 1299.99 },
    ];
    
    let affordable: Vec<Product> = tokio_stream::iter(products)
        .filter(|p| futures::future::ready(p.price < 100.0))
        .collect()
        .await;
    
    println!("Affordable products:");
    for product in affordable {
        println!("  {} - ${:.2}", product.name, product.price);
    }
}
```

Output:
```
Affordable products:
  Mouse - $25.99
  Keyboard - $89.99
  Cable - $9.99
```

## Advanced: `filter_map` Combinator

For filtering and mapping in one step:

```rust
use tokio_stream;
use futures::StreamExt;

#[tokio::main]
async fn main() {
    let results: Vec<i32> = tokio_stream::iter(1..=10)
        .filter_map(|x| {
            if x % 2 == 0 {
                futures::future::ready(Some(x * x))  // Keep and square
            } else {
                futures::future::ready(None)          // Skip
            }
        })
        .collect()
        .await;
    
    println!("Results: {:?}", results);
}
```

Output: `Results: [4, 16, 36, 64, 100]`

**Benefits:**
- More concise than separate `filter` and `map`
- Slightly more efficient (single pass)
- Common pattern for Result/Option handling

## Performance Considerations

### Lazy Evaluation

```rust
// Filter is not executed here
let stream = tokio_stream::iter(1..=1000000)
    .filter(|x| {
        println!("Testing {}", x);  // Won't print yet!
        futures::future::ready(x % 2 == 0)
    });

// Filter executes as values are pulled
let evens: Vec<i32> = stream.take(5).collect().await;
// Only tests values until 5 evens are found
```

### Efficient Chaining

```rust
// ✅ Good: Single pass, efficient
tokio_stream::iter(data)
    .filter(|x| futures::future::ready(x.is_valid()))
    .map(|x| x.process())
    .collect()
    .await

// ❌ Bad: Multiple passes, creates intermediate vectors
let filtered: Vec<_> = tokio_stream::iter(data)
    .filter(|x| futures::future::ready(x.is_valid()))
    .collect()
    .await;

let processed: Vec<_> = tokio_stream::iter(filtered)
    .map(|x| x.process())
    .collect()
    .await;
```

## Common Pitfalls

### Pitfall 1: Forgetting `futures::future::ready`

```rust
// ❌ Won't compile
.filter(|x| x % 2 == 0)

// ✅ Correct
.filter(|x| futures::future::ready(x % 2 == 0))
```

### Pitfall 2: Borrowing Issues

```rust
// ❌ May have issues with borrowing
.filter(|x| futures::future::ready(x % 2 == 0))  // x is &i32

// ✅ Dereference if needed
.filter(|x| futures::future::ready(*x % 2 == 0))
```

### Pitfall 3: Side Effects in Filter

```rust
// ❌ Bad: Side effects in filter predicate
.filter(|x| {
    println!("Testing {}", x);  // Side effect!
    futures::future::ready(*x % 2 == 0)
})

// ✅ Better: Use inspect for side effects
.inspect(|x| println!("Processing {}", x))
.filter(|x| futures::future::ready(*x % 2 == 0))
```

## Best Practices

### 1. Keep Predicates Simple

```rust
// ✅ Good: Simple, clear predicate
.filter(|x| futures::future::ready(*x > 0))

// ❌ Bad: Complex logic in predicate
.filter(|x| futures::future::ready({
    let result = complex_calculation(*x);
    result > 0 && result < 100 && result % 2 == 0
}))

// ✅ Better: Extract to function
fn is_valid(x: i32) -> bool {
    let result = complex_calculation(x);
    result > 0 && result < 100 && result % 2 == 0
}

.filter(|x| futures::future::ready(is_valid(*x)))
```

### 2. Use `filter_map` When Appropriate

```rust
// Instead of filter + map
.filter(|x| futures::future::ready(x.is_some()))
.map(|x| x.unwrap())

// Use filter_map
.filter_map(|x| futures::future::ready(x))
```

### 3. Consider Order of Operations

```rust
// ✅ Better: Filter first (reduces work)
.filter(|x| futures::future::ready(*x > 0))
.map(|x| expensive_operation(x))

// ❌ Worse: Map first (more work)
.map(|x| expensive_operation(x))
.filter(|x| futures::future::ready(*x > 0))
```

## Summary

The `filter` combinator is essential for stream processing:

1. **Selective processing**: Only keep values matching criteria
2. **Lazy evaluation**: Filtering happens as values are consumed
3. **Composable**: Chains with other stream combinators
4. **Type-safe**: Maintains type information
5. **Async-ready**: Supports both sync and async predicates

### Basic Pattern

```rust
let filtered = tokio_stream::iter(data)
    .filter(|item| futures::future::ready(condition(item)))
    .collect()
    .await;
```

### When to Use `filter`

- Removing invalid/unwanted values
- Selecting subset of data
- Validation filtering
- Conditional processing
- Building data pipelines

The `filter` combinator, combined with other stream operations like `map`, `then`, and `take`, enables powerful and expressive data processing in async Rust applications.