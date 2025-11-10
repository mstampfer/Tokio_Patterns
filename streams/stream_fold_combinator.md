# Aggregating Stream Values with `fold`

## Overview

This code demonstrates how to use the **`fold` combinator** to aggregate all values in a stream into a single result. The `fold` operation is a powerful reduction technique that processes each stream element sequentially, accumulating a result by applying a function that combines the current accumulator with each new value.

## Complete Code

```rust
use tokio_stream as stream;
use futures::StreamExt;

#[tokio::main]
async fn main() {
    let sum = stream::iter(1..=10)
        .fold(0, |acc, x| async move { acc + x })
        .await;
    
    println!("Sum: {}", sum);
}
```

## Cargo.toml

```toml
[package]
name = "stream-fold-example"
version = "0.1.0"
edition = "2021"

[dependencies]
tokio = { version = "1", features = ["full"] }
tokio-stream = "0.1"
futures = "0.3"
```

## Expected Output

```
Sum: 55
```

## What is the `fold` Combinator?

The `fold` combinator consumes a stream and reduces it to a single value by repeatedly applying an accumulator function. It's the async equivalent of the iterator `fold` method.

### Function Signature

```rust
fn fold<B, F, Fut>(self, init: B, f: F) -> Fold<Self, F, Fut, B>
where
    F: FnMut(B, Self::Item) -> Fut,
    Fut: Future<Output = B>,
```

**Parameters:**
- `init`: The initial accumulator value
- `f`: A closure that takes `(accumulator, item)` and returns a `Future<Output = accumulator>`

**Returns:**
- A `Future` that resolves to the final accumulated value

## How This Code Works

### Step 1: Create a Stream

```rust
let sum = stream::iter(1..=10)
```

Creates a stream with values from 1 to 10:
```
Stream: [1, 2, 3, 4, 5, 6, 7, 8, 9, 10]
```

### Step 2: Apply `fold` with Accumulator Function

```rust
    .fold(0, |acc, x| async move { acc + x })
```

**What happens:**

1. **`0`**: Initial accumulator value (starting sum)
2. **`|acc, x|`**: Closure parameters
   - `acc`: Current accumulated value
   - `x`: Current stream item
3. **`async move { acc + x }`**: Async block that adds the item to the accumulator
4. Returns a new accumulator value for the next iteration

### Step 3: Await the Result

```rust
    .await;
```

The `fold` operation returns a `Future`, so we must `.await` it to get the final result.

### Step 4: Print the Result

```rust
println!("Sum: {}", sum);
```

Prints: `Sum: 55`

## Visual Execution Flow

```
Initial State:
accumulator = 0

Stream: [1, 2, 3, 4, 5, 6, 7, 8, 9, 10]

Iteration 1:
  acc = 0, x = 1
  new_acc = 0 + 1 = 1
  ↓
Iteration 2:
  acc = 1, x = 2
  new_acc = 1 + 2 = 3
  ↓
Iteration 3:
  acc = 3, x = 3
  new_acc = 3 + 3 = 6
  ↓
Iteration 4:
  acc = 6, x = 4
  new_acc = 6 + 4 = 10
  ↓
Iteration 5:
  acc = 10, x = 5
  new_acc = 10 + 5 = 15
  ↓
Iteration 6:
  acc = 15, x = 6
  new_acc = 15 + 6 = 21
  ↓
Iteration 7:
  acc = 21, x = 7
  new_acc = 21 + 7 = 28
  ↓
Iteration 8:
  acc = 28, x = 8
  new_acc = 28 + 8 = 36
  ↓
Iteration 9:
  acc = 36, x = 9
  new_acc = 36 + 9 = 45
  ↓
Iteration 10:
  acc = 45, x = 10
  new_acc = 45 + 10 = 55
  ↓
Final Result: 55
```

## Detailed Step-by-Step Trace

```
Step  | Accumulator | Stream Item | Operation      | New Accumulator
------|-------------|-------------|----------------|----------------
Start | 0           | -           | Initial        | 0
1     | 0           | 1           | 0 + 1          | 1
2     | 1           | 2           | 1 + 2          | 3
3     | 3           | 3           | 3 + 3          | 6
4     | 6           | 4           | 6 + 4          | 10
5     | 10          | 5           | 10 + 5         | 15
6     | 15          | 6           | 15 + 6         | 21
7     | 21          | 7           | 21 + 7         | 28
8     | 28          | 8           | 28 + 8         | 36
9     | 36          | 9           | 36 + 9         | 45
10    | 45          | 10          | 45 + 10        | 55
End   | 55          | -           | Stream empty   | 55 (final)
```

## Visual Data Flow Diagram

```
┌─────────────────────────────────────────────────────────┐
│           Stream: [1, 2, 3, 4, 5, 6, 7, 8, 9, 10]       │
└───────────────────────┬─────────────────────────────────┘
                        │
                        ▼
                 .fold(0, |acc, x| async move { acc + x })
                        │
        ┌───────────────┴───────────────┐
        │  Accumulation Process:        │
        │                               │
        │  acc = 0                      │
        │  ├─ + 1 → acc = 1             │
        │  ├─ + 2 → acc = 3             │
        │  ├─ + 3 → acc = 6             │
        │  ├─ + 4 → acc = 10            │
        │  ├─ + 5 → acc = 15            │
        │  ├─ + 6 → acc = 21            │
        │  ├─ + 7 → acc = 28            │
        │  ├─ + 8 → acc = 36            │
        │  ├─ + 9 → acc = 45            │
        │  └─ + 10 → acc = 55           │
        │                               │
        └───────────────┬───────────────┘
                        │
                        ▼
                   .await
                        │
                        ▼
                  Final Result: 55
```

## Why `async move` is Required

### The Problem with Direct Values

```rust
// ❌ This won't compile!
.fold(0, |acc, x| acc + x)
// Error: expected Future, found i32
```

### The Solution: Async Block

```rust
// ✅ This works!
.fold(0, |acc, x| async move { acc + x })
```

**Explanation:**
- Stream's `fold` expects a closure that returns a `Future<Output = Accumulator>`
- `async move { ... }` creates a future
- Even for simple operations, you must wrap in async block
- This allows `fold` to support async operations in the accumulator function

## Comparison: Iterator vs Stream Fold

### Synchronous Iterator

```rust
let sum: i32 = (1..=10)
    .fold(0, |acc, x| acc + x);  // Direct value

println!("Sum: {}", sum);
```

**Characteristics:**
- Returns value directly
- No `.await` needed
- Synchronous execution
- Simple closure

### Async Stream

```rust
let sum = tokio_stream::iter(1..=10)
    .fold(0, |acc, x| async move { acc + x })  // Returns Future
    .await;

println!("Sum: {}", sum);
```

**Characteristics:**
- Returns `Future`
- Requires `.await`
- Async execution
- Closure returns future

## Common Folding Patterns

### Pattern 1: Summing Numbers

```rust
use tokio_stream as stream;
use futures::StreamExt;

#[tokio::main]
async fn main() {
    let sum = stream::iter(1..=100)
        .fold(0, |acc, x| async move { acc + x })
        .await;
    
    println!("Sum: {}", sum);
}
```

Output: `Sum: 5050`

### Pattern 2: Finding Maximum

```rust
use tokio_stream as stream;
use futures::StreamExt;

#[tokio::main]
async fn main() {
    let max = stream::iter(vec![3, 7, 2, 9, 1, 5])
        .fold(i32::MIN, |acc, x| async move { acc.max(x) })
        .await;
    
    println!("Max: {}", max);
}
```

Output: `Max: 9`

### Pattern 3: Finding Minimum

```rust
use tokio_stream as stream;
use futures::StreamExt;

#[tokio::main]
async fn main() {
    let min = stream::iter(vec![3, 7, 2, 9, 1, 5])
        .fold(i32::MAX, |acc, x| async move { acc.min(x) })
        .await;
    
    println!("Min: {}", min);
}
```

Output: `Min: 1`

### Pattern 4: Counting Elements

```rust
use tokio_stream as stream;
use futures::StreamExt;

#[tokio::main]
async fn main() {
    let count = stream::iter(vec!["a", "b", "c", "d", "e"])
        .fold(0, |acc, _| async move { acc + 1 })
        .await;
    
    println!("Count: {}", count);
}
```

Output: `Count: 5`

### Pattern 5: Product of Numbers

```rust
use tokio_stream as stream;
use futures::StreamExt;

#[tokio::main]
async fn main() {
    let product = stream::iter(1..=5)
        .fold(1, |acc, x| async move { acc * x })
        .await;
    
    println!("Product: {}", product);
}
```

Output: `Product: 120` (factorial of 5)

### Pattern 6: Building a String

```rust
use tokio_stream as stream;
use futures::StreamExt;

#[tokio::main]
async fn main() {
    let result = stream::iter(vec!["Hello", " ", "World", "!"])
        .fold(String::new(), |acc, s| async move {
            acc + s
        })
        .await;
    
    println!("Result: {}", result);
}
```

Output: `Result: Hello World!`

### Pattern 7: Building a Vector

```rust
use tokio_stream as stream;
use futures::StreamExt;

#[tokio::main]
async fn main() {
    let doubled = stream::iter(1..=5)
        .fold(Vec::new(), |mut acc, x| async move {
            acc.push(x * 2);
            acc
        })
        .await;
    
    println!("Doubled: {:?}", doubled);
}
```

Output: `Doubled: [2, 4, 6, 8, 10]`

## Practical Examples

### Example 1: Computing Statistics

```rust
use tokio_stream as stream;
use futures::StreamExt;

#[derive(Debug, Default)]
struct Stats {
    count: usize,
    sum: i32,
    min: i32,
    max: i32,
}

impl Stats {
    fn average(&self) -> f64 {
        if self.count == 0 {
            0.0
        } else {
            self.sum as f64 / self.count as f64
        }
    }
}

#[tokio::main]
async fn main() {
    let numbers = vec![5, 12, 3, 8, 15, 7, 10];
    
    let stats = stream::iter(numbers)
        .fold(Stats::default(), |mut acc, x| async move {
            acc.count += 1;
            acc.sum += x;
            acc.min = if acc.count == 1 { x } else { acc.min.min(x) };
            acc.max = if acc.count == 1 { x } else { acc.max.max(x) };
            acc
        })
        .await;
    
    println!("Statistics:");
    println!("  Count: {}", stats.count);
    println!("  Sum: {}", stats.sum);
    println!("  Average: {:.2}", stats.average());
    println!("  Min: {}", stats.min);
    println!("  Max: {}", stats.max);
}
```

Output:
```
Statistics:
  Count: 7
  Sum: 60
  Average: 8.57
  Min: 3
  Max: 15
```

### Example 2: Grouping by Category

```rust
use tokio_stream as stream;
use futures::StreamExt;
use std::collections::HashMap;

#[derive(Debug)]
struct Item {
    category: String,
    value: i32,
}

#[tokio::main]
async fn main() {
    let items = vec![
        Item { category: "A".to_string(), value: 10 },
        Item { category: "B".to_string(), value: 20 },
        Item { category: "A".to_string(), value: 15 },
        Item { category: "C".to_string(), value: 5 },
        Item { category: "B".to_string(), value: 25 },
    ];
    
    let grouped = stream::iter(items)
        .fold(HashMap::new(), |mut acc, item| async move {
            *acc.entry(item.category).or_insert(0) += item.value;
            acc
        })
        .await;
    
    println!("Grouped totals:");
    for (category, total) in grouped {
        println!("  {}: {}", category, total);
    }
}
```

Output:
```
Grouped totals:
  A: 25
  B: 45
  C: 5
```

### Example 3: Async Operations in Fold

```rust
use tokio_stream as stream;
use futures::StreamExt;
use tokio::time::{sleep, Duration};

async fn fetch_price(product_id: i32) -> f64 {
    // Simulate async API call
    sleep(Duration::from_millis(10)).await;
    product_id as f64 * 9.99
}

#[tokio::main]
async fn main() {
    let product_ids = vec![1, 2, 3, 4, 5];
    
    let total_cost = stream::iter(product_ids)
        .fold(0.0, |acc, id| async move {
            let price = fetch_price(id).await;
            println!("Product {}: ${:.2}", id, price);
            acc + price
        })
        .await;
    
    println!("\nTotal cost: ${:.2}", total_cost);
}
```

Output:
```
Product 1: $9.99
Product 2: $19.98
Product 3: $29.97
Product 4: $39.96
Product 5: $49.95

Total cost: $149.85
```

### Example 4: Building Complex Data Structures

```rust
use tokio_stream as stream;
use futures::StreamExt;

#[derive(Debug)]
struct Report {
    total_sales: f64,
    transaction_count: usize,
    items_sold: Vec<String>,
}

#[derive(Debug)]
struct Transaction {
    amount: f64,
    items: Vec<String>,
}

#[tokio::main]
async fn main() {
    let transactions = vec![
        Transaction { amount: 29.99, items: vec!["Book".to_string()] },
        Transaction { amount: 49.99, items: vec!["Shirt".to_string(), "Hat".to_string()] },
        Transaction { amount: 19.99, items: vec!["Mug".to_string()] },
    ];
    
    let report = stream::iter(transactions)
        .fold(
            Report {
                total_sales: 0.0,
                transaction_count: 0,
                items_sold: Vec::new(),
            },
            |mut acc, transaction| async move {
                acc.total_sales += transaction.amount;
                acc.transaction_count += 1;
                acc.items_sold.extend(transaction.items);
                acc
            },
        )
        .await;
    
    println!("Sales Report:");
    println!("  Total Sales: ${:.2}", report.total_sales);
    println!("  Transactions: {}", report.transaction_count);
    println!("  Items Sold: {:?}", report.items_sold);
}
```

Output:
```
Sales Report:
  Total Sales: $99.97
  Transactions: 3
  Items Sold: ["Book", "Shirt", "Hat", "Mug"]
```

### Example 5: Error Accumulation

```rust
use tokio_stream as stream;
use futures::StreamExt;

#[derive(Debug)]
struct ValidationResult {
    id: i32,
    is_valid: bool,
    error: Option<String>,
}

#[derive(Debug)]
struct Summary {
    valid_count: usize,
    invalid_count: usize,
    errors: Vec<String>,
}

#[tokio::main]
async fn main() {
    let results = vec![
        ValidationResult { id: 1, is_valid: true, error: None },
        ValidationResult { id: 2, is_valid: false, error: Some("Invalid format".to_string()) },
        ValidationResult { id: 3, is_valid: true, error: None },
        ValidationResult { id: 4, is_valid: false, error: Some("Missing field".to_string()) },
    ];
    
    let summary = stream::iter(results)
        .fold(
            Summary {
                valid_count: 0,
                invalid_count: 0,
                errors: Vec::new(),
            },
            |mut acc, result| async move {
                if result.is_valid {
                    acc.valid_count += 1;
                } else {
                    acc.invalid_count += 1;
                    if let Some(error) = result.error {
                        acc.errors.push(format!("ID {}: {}", result.id, error));
                    }
                }
                acc
            },
        )
        .await;
    
    println!("Validation Summary:");
    println!("  Valid: {}", summary.valid_count);
    println!("  Invalid: {}", summary.invalid_count);
    println!("  Errors:");
    for error in summary.errors {
        println!("    - {}", error);
    }
}
```

Output:
```
Validation Summary:
  Valid: 2
  Invalid: 2
  Errors:
    - ID 2: Invalid format
    - ID 4: Missing field
```

## Combining `fold` with Other Combinators

### Filter then Fold

```rust
use tokio_stream as stream;
use futures::StreamExt;

#[tokio::main]
async fn main() {
    let sum_of_evens = stream::iter(1..=10)
        .filter(|x| futures::future::ready(x % 2 == 0))
        .fold(0, |acc, x| async move { acc + x })
        .await;
    
    println!("Sum of evens: {}", sum_of_evens);
}
```

Output: `Sum of evens: 30`

### Map then Fold

```rust
let sum_of_squares = stream::iter(1..=5)
    .map(|x| x * x)
    .fold(0, |acc, x| async move { acc + x })
    .await;

// Sum of squares: 55 (1 + 4 + 9 + 16 + 25)
```

### Take then Fold

```rust
let sum_of_first_five = stream::iter(1..=100)
    .take(5)
    .fold(0, |acc, x| async move { acc + x })
    .await;

// Sum of first five: 15 (1 + 2 + 3 + 4 + 5)
```

## Performance Considerations

### Sequential Processing

```rust
// Fold processes items sequentially, one at a time
.fold(0, |acc, x| async move {
    expensive_operation(x).await;  // Each waits for previous
    acc + x
})
```

**Characteristics:**
- Items processed in order
- No parallelism
- Preserves accumulator state

### Memory Efficiency

```rust
// ✅ Good: Accumulates without intermediate collections
.fold(0, |acc, x| async move { acc + x })

// ❌ Bad: Creates intermediate vector
.collect().await.iter().sum()
```

## Common Pitfalls

### Pitfall 1: Forgetting `async move`

```rust
// ❌ Won't compile
.fold(0, |acc, x| acc + x)

// ✅ Correct
.fold(0, |acc, x| async move { acc + x })
```

### Pitfall 2: Forgetting `.await`

```rust
// ❌ sum is a Future, not i32
let sum = stream::iter(1..=10)
    .fold(0, |acc, x| async move { acc + x });

// ✅ Correct
let sum = stream::iter(1..=10)
    .fold(0, |acc, x| async move { acc + x })
    .await;
```

### Pitfall 3: Mutating Without Returning

```rust
// ❌ Bad: Doesn't return accumulator
.fold(Vec::new(), |mut acc, x| async move {
    acc.push(x);
    // Missing: acc
})

// ✅ Correct: Returns accumulator
.fold(Vec::new(), |mut acc, x| async move {
    acc.push(x);
    acc
})
```

## Best Practices

### 1. Choose Appropriate Initial Value

```rust
// Sum: start with 0
.fold(0, |acc, x| async move { acc + x })

// Product: start with 1
.fold(1, |acc, x| async move { acc * x })

// Min: start with MAX
.fold(i32::MAX, |acc, x| async move { acc.min(x) })

// Max: start with MIN
.fold(i32::MIN, |acc, x| async move { acc.max(x) })
```

### 2. Use Mutable Accumulators for Complex Types

```rust
.fold(Vec::new(), |mut acc, x| async move {
    acc.push(x);
    acc  // Don't forget to return!
})
```

### 3. Consider `collect()` for Simple Cases

```rust
// If just collecting, use collect()
let vec: Vec<i32> = stream.collect().await;

// Use fold for aggregation
let sum: i32 = stream.fold(0, |acc, x| async move { acc + x }).await;
```

### 4. Extract Complex Logic

```rust
async fn accumulate(acc: Stats, value: i32) -> Stats {
    // Complex accumulation logic
}

.fold(Stats::default(), |acc, x| accumulate(acc, x))
```

## Summary

The `fold` combinator is a powerful tool for stream aggregation:

1. **Reduces stream to single value**: Consumes all items, produces one result
2. **Sequential processing**: Items processed in order with accumulator
3. **Requires async closure**: Must return `Future<Output = Accumulator>`
4. **Flexible accumulation**: Supports any accumulator type and operation
5. **Lazy evaluation**: Only executes when awaited

### Basic Pattern

```rust
let result = tokio_stream::iter(items)
    .fold(initial_value, |accumulator, item| async move {
        // Combine accumulator with item
        new_accumulator
    })
    .await;
```

### When to Use `fold`

- **Summing/aggregating**: Computing totals, averages, statistics
- **Finding extremes**: Maximum, minimum values
- **Building collections**: Accumulating into Vec, HashMap, etc.
- **Combining results**: Merging multiple items into one
- **Complex reductions**: Any operation that combines all items

The `fold` combinator enables powerful data reduction operations in async streams, making it essential for tasks that require aggregating stream data into a single meaningful result.