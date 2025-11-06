# Running Multiple Futures Concurrently with tokio::join!

## Complete Code

```rust
use tokio::time::{sleep, Duration};

async fn task1() -> i32 {
    sleep(Duration::from_millis(100)).await;
    1
}

async fn task2() -> i32 {
    sleep(Duration::from_millis(50)).await;
    2
}

#[tokio::main]
async fn main() {
    // Run both tasks concurrently
    let (result1, result2) = tokio::join!(task1(), task2());
    
    println!("Results: {} and {}", result1, result2);
}
```

## Output

```
Results: 1 and 2
```

**Execution time:** ~100ms (not 150ms!)

## Understanding tokio::join!

### What is tokio::join!?

`tokio::join!` is a macro that runs multiple futures concurrently and waits for all of them to complete:

```rust
let (result1, result2) = tokio::join!(future1, future2);
```

**Key characteristics:**
- Runs all futures **concurrently** (at the same time)
- Waits for **all** futures to complete
- Returns results in a tuple, in the same order as arguments
- Efficient - doesn't spawn separate tasks (runs on same task)

### Concurrent vs Sequential Execution

**Sequential execution (without join!):**

```rust
let result1 = task1().await;  // Wait 100ms
let result2 = task2().await;  // Wait 50ms more
// Total time: 150ms
```

**Concurrent execution (with join!):**

```rust
let (result1, result2) = tokio::join!(task1(), task2());
// Both run at the same time
// Total time: 100ms (the longest task)
```

## How tokio::join! Works

### The Concurrent Execution Model

When you call `tokio::join!`, it:

1. **Starts all futures immediately**
2. **Polls them in a round-robin fashion**
3. **Waits for all to complete**
4. **Returns all results together**

### Visual Timeline

```
Time (ms)    task1 (100ms)              task2 (50ms)
────────────────────────────────────────────────────────
0            ▼ Start                    ▼ Start
             │ Sleeping...              │ Sleeping...
             │                          │
50           │                          ✓ Complete (returns 2)
             │                          │ (waits for task1)
             │                          │
100          ✓ Complete (returns 1)     │
             │                          │
             └──────────────────────────┘
             Both complete, join! returns (1, 2)
```

**Key observation:** Even though `task2` finishes at 50ms, `join!` waits for `task1` to complete at 100ms before returning.

### Step-by-Step Execution

#### Step 1: Call tokio::join!

```rust
let (result1, result2) = tokio::join!(task1(), task2());
```

**What happens:**
- `task1()` is called, creating a future (doesn't execute yet)
- `task2()` is called, creating a future (doesn't execute yet)
- `tokio::join!` begins polling both futures

#### Step 2: Start Polling

The `tokio::join!` macro roughly expands to something like:

```rust
async {
    let mut fut1 = task1();
    let mut fut2 = task2();
    
    // Pin both futures
    let mut fut1 = std::pin::pin!(fut1);
    let mut fut2 = std::pin::pin!(fut2);
    
    let mut result1 = None;
    let mut result2 = None;
    
    // Poll both futures until complete
    loop {
        // Poll fut1 if not complete
        if result1.is_none() {
            if let Poll::Ready(val) = fut1.poll(...) {
                result1 = Some(val);
            }
        }
        
        // Poll fut2 if not complete
        if result2.is_none() {
            if let Poll::Ready(val) = fut2.poll(...) {
                result2 = Some(val);
            }
        }
        
        // If both complete, break
        if result1.is_some() && result2.is_some() {
            break;
        }
    }
    
    (result1.unwrap(), result2.unwrap())
}
```

This is simplified, but shows the core idea: poll both futures until both complete.

#### Step 3: Concurrent Sleep

**At t=0ms:**
- `task1` starts sleeping (100ms)
- `task2` starts sleeping (50ms)
- Both return `Poll::Pending`
- The runtime yields control to other tasks

**At t=50ms:**
- `task2`'s sleep completes
- `task2` is polled again
- Returns `Poll::Ready(2)`
- `result2` is stored
- `task1` is still sleeping

**At t=100ms:**
- `task1`'s sleep completes
- `task1` is polled again
- Returns `Poll::Ready(1)`
- `result1` is stored
- Both tasks complete

#### Step 4: Return Results

```rust
let (result1, result2) = (1, 2);
println!("Results: {} and {}", result1, result2);
```

**Output:** `Results: 1 and 2`

## Detailed Example Breakdown

### The Task Functions

```rust
async fn task1() -> i32 {
    sleep(Duration::from_millis(100)).await;
    1
}
```

**What this does:**
1. Creates an async function that returns a future
2. When awaited, sleeps for 100ms
3. Returns the value `1`

**Important:** The sleep is non-blocking - it yields control back to the runtime, allowing other tasks to run.

```rust
async fn task2() -> i32 {
    sleep(Duration::from_millis(50)).await;
    2
}
```

**What this does:**
1. Similar to task1
2. Sleeps for only 50ms (half as long)
3. Returns the value `2`

### The Join Operation

```rust
let (result1, result2) = tokio::join!(task1(), task2());
```

**Type of results:**
- `result1: i32` (from task1)
- `result2: i32` (from task2)
- The tuple: `(i32, i32)`

**Order guarantee:**
- Results are returned in the order of arguments
- `result1` comes from the first argument (`task1()`)
- `result2` comes from the second argument (`task2()`)
- This is true regardless of which task completes first

## Comparison with Other Approaches

### Approach 1: Sequential Execution

```rust
let result1 = task1().await;  // Takes 100ms
let result2 = task2().await;  // Takes 50ms
println!("Results: {} and {}", result1, result2);
// Total time: 150ms
```

**Pros:**
- Simple and straightforward
- Easy to understand

**Cons:**
- ❌ Slower - tasks run one after another
- ❌ Wastes time when tasks could run concurrently

### Approach 2: Using tokio::join! (Our Example)

```rust
let (result1, result2) = tokio::join!(task1(), task2());
println!("Results: {} and {}", result1, result2);
// Total time: 100ms (the longest task)
```

**Pros:**
- ✅ Fast - tasks run concurrently
- ✅ Simple syntax
- ✅ No extra task spawning overhead
- ✅ Waits for all tasks to complete

**Cons:**
- Must wait for all tasks (can't get early results)

### Approach 3: Spawning Tasks with tokio::spawn

```rust
let handle1 = tokio::spawn(task1());
let handle2 = tokio::spawn(task2());

let result1 = handle1.await.unwrap();
let result2 = handle2.await.unwrap();
println!("Results: {} and {}", result1, result2);
// Total time: ~100ms
```

**Pros:**
- ✅ Tasks can run on different threads
- ✅ Good for CPU-bound work

**Cons:**
- ❌ More overhead (spawning tasks)
- ❌ Must handle `JoinError`
- ❌ More complex

### Approach 4: Using tokio::select! (First to Complete)

```rust
tokio::select! {
    result = task1() => println!("Task 1 finished first: {}", result),
    result = task2() => println!("Task 2 finished first: {}", result),
}
// Returns as soon as ONE completes
// The other task is cancelled!
```

**Pros:**
- ✅ Fast - returns as soon as one completes
- ✅ Good for racing tasks

**Cons:**
- ❌ Only gets one result
- ❌ Cancels other tasks

## tokio::join! Features

### Supports Multiple Futures

```rust
// Join 2 futures:
let (r1, r2) = tokio::join!(fut1, fut2);

// Join 3 futures:
let (r1, r2, r3) = tokio::join!(fut1, fut2, fut3);

// Join many futures:
let (r1, r2, r3, r4, r5) = tokio::join!(fut1, fut2, fut3, fut4, fut5);

// Up to 64 futures!
```

### Works with Different Return Types

```rust
async fn get_number() -> i32 { 42 }
async fn get_string() -> String { "Hello".to_string() }
async fn get_bool() -> bool { true }

let (num, text, flag) = tokio::join!(
    get_number(),
    get_string(),
    get_bool()
);

// num: i32 = 42
// text: String = "Hello"
// flag: bool = true
```

### Handles Errors Gracefully

```rust
async fn may_fail1() -> Result<i32, &'static str> {
    Ok(1)
}

async fn may_fail2() -> Result<i32, &'static str> {
    Err("Failed")
}

let (result1, result2) = tokio::join!(may_fail1(), may_fail2());

// result1: Result<i32, &str> = Ok(1)
// result2: Result<i32, &str> = Err("Failed")

// Both complete even if one fails!
```

## Practical Examples

### Example 1: Parallel API Calls

```rust
use tokio::time::{sleep, Duration};

async fn fetch_user(id: u32) -> String {
    sleep(Duration::from_millis(100)).await;
    format!("User {}", id)
}

async fn fetch_posts(user_id: u32) -> Vec<String> {
    sleep(Duration::from_millis(150)).await;
    vec![format!("Post 1 by user {}", user_id)]
}

async fn fetch_comments(user_id: u32) -> Vec<String> {
    sleep(Duration::from_millis(80)).await;
    vec![format!("Comment 1 by user {}", user_id)]
}

#[tokio::main]
async fn main() {
    let start = std::time::Instant::now();
    
    let (user, posts, comments) = tokio::join!(
        fetch_user(1),
        fetch_posts(1),
        fetch_comments(1)
    );
    
    println!("User: {}", user);
    println!("Posts: {:?}", posts);
    println!("Comments: {:?}", comments);
    println!("Took: {:?}", start.elapsed());
    // Took: ~150ms (not 330ms!)
}
```

### Example 2: Database Queries

```rust
async fn query_users() -> Vec<String> {
    sleep(Duration::from_millis(200)).await;
    vec!["Alice".to_string(), "Bob".to_string()]
}

async fn query_products() -> Vec<String> {
    sleep(Duration::from_millis(150)).await;
    vec!["Widget".to_string(), "Gadget".to_string()]
}

async fn query_orders() -> Vec<String> {
    sleep(Duration::from_millis(100)).await;
    vec!["Order1".to_string(), "Order2".to_string()]
}

#[tokio::main]
async fn main() {
    // Run all queries concurrently
    let (users, products, orders) = tokio::join!(
        query_users(),
        query_products(),
        query_orders()
    );
    
    println!("Users: {}", users.len());
    println!("Products: {}", products.len());
    println!("Orders: {}", orders.len());
    // Total time: ~200ms (longest query)
}
```

### Example 3: File Operations

```rust
use tokio::fs;

async fn read_config() -> String {
    sleep(Duration::from_millis(50)).await;
    "config_data".to_string()
}

async fn read_users() -> String {
    sleep(Duration::from_millis(75)).await;
    "user_data".to_string()
}

async fn read_logs() -> String {
    sleep(Duration::from_millis(100)).await;
    "log_data".to_string()
}

#[tokio::main]
async fn main() {
    let (config, users, logs) = tokio::join!(
        read_config(),
        read_users(),
        read_logs()
    );
    
    println!("Loaded config: {} bytes", config.len());
    println!("Loaded users: {} bytes", users.len());
    println!("Loaded logs: {} bytes", logs.len());
    // All files read concurrently!
}
```

## Performance Analysis

### Time Complexity

**Sequential execution:**
```
Total Time = T1 + T2 + T3 + ... + Tn
```

For our example: 100ms + 50ms = 150ms

**Concurrent execution with tokio::join!:**
```
Total Time = max(T1, T2, T3, ..., Tn)
```

For our example: max(100ms, 50ms) = 100ms

### Speedup Calculation

```
Speedup = Sequential Time / Concurrent Time
Speedup = 150ms / 100ms = 1.5x faster
```

### Real-World Benchmarks

Let's compare different numbers of tasks:

| Tasks | Sequential Time | Concurrent Time | Speedup |
|-------|----------------|-----------------|---------|
| 2 (100ms, 50ms) | 150ms | 100ms | 1.5x |
| 3 (100ms, 100ms, 100ms) | 300ms | 100ms | 3x |
| 5 (100ms each) | 500ms | 100ms | 5x |
| 10 (100ms each) | 1000ms | 100ms | 10x |

**Key insight:** The more independent I/O-bound tasks you have, the greater the speedup!

## Common Patterns

### Pattern 1: Join with Error Handling

```rust
let (r1, r2) = tokio::join!(
    async {
        task1().await.map_err(|e| format!("Task1 error: {}", e))
    },
    async {
        task2().await.map_err(|e| format!("Task2 error: {}", e))
    }
);

match (r1, r2) {
    (Ok(v1), Ok(v2)) => println!("Success: {}, {}", v1, v2),
    (Err(e), _) | (_, Err(e)) => println!("Error: {}", e),
}
```

### Pattern 2: Join with Timeouts

```rust
use tokio::time::timeout;

let (r1, r2) = tokio::join!(
    timeout(Duration::from_secs(5), task1()),
    timeout(Duration::from_secs(5), task2())
);

// Both tasks have individual timeouts
```

### Pattern 3: Join with Default Values

```rust
let (r1, r2) = tokio::join!(task1(), task2());

let result1 = r1.unwrap_or(0);  // Default to 0 if error
let result2 = r2.unwrap_or(0);

println!("Results: {}, {}", result1, result2);
```

## When to Use tokio::join!

### ✅ Use tokio::join! When:

1. **Multiple independent I/O operations**
   - API calls, database queries, file operations
   
2. **You need all results**
   - Can't proceed until all tasks complete
   
3. **Tasks are lightweight**
   - Don't need separate OS threads
   
4. **Fixed number of tasks**
   - Know at compile time how many tasks to run

5. **Tasks have similar completion times**
   - No one task is dramatically slower

### ❌ Don't Use tokio::join! When:

1. **Tasks depend on each other**
   - Use sequential `.await` instead
   
2. **Only need first result**
   - Use `tokio::select!` instead
   
3. **CPU-bound work**
   - Use `tokio::spawn` to run on separate threads
   
4. **Dynamic number of tasks**
   - Use `futures::future::join_all` or `FuturesUnordered`
   
5. **Need cancellation of other tasks**
   - Use `tokio::select!` instead

## Comparison Table

| Feature | tokio::join! | Sequential | tokio::spawn | tokio::select! |
|---------|-------------|-----------|--------------|----------------|
| **Execution** | Concurrent | Sequential | Parallel (threads) | Concurrent (races) |
| **Waits for all** | ✅ Yes | ✅ Yes | ✅ Yes | ❌ No (first only) |
| **Returns all results** | ✅ Yes | ✅ Yes | ✅ Yes | ❌ No (one only) |
| **Overhead** | Low | Lowest | High | Low |
| **Thread pool** | Single task | Single task | Multiple tasks | Single task |
| **Cancellation** | None | None | Manual | Automatic |

## Advanced: Under the Hood

### What the Macro Expands To

```rust
// Source code:
let (r1, r2) = tokio::join!(task1(), task2());

// Roughly expands to:
let (r1, r2) = async {
    let mut fut1 = task1();
    let mut fut2 = task2();
    
    pin!(fut1);
    pin!(fut2);
    
    let mut result1 = None;
    let mut result2 = None;
    
    poll_fn(|cx| {
        if result1.is_none() {
            if let Poll::Ready(v) = fut1.as_mut().poll(cx) {
                result1 = Some(v);
            }
        }
        
        if result2.is_none() {
            if let Poll::Ready(v) = fut2.as_mut().poll(cx) {
                result2 = Some(v);
            }
        }
        
        if result1.is_some() && result2.is_some() {
            Poll::Ready((result1.unwrap(), result2.unwrap()))
        } else {
            Poll::Pending
        }
    }).await
}.await;
```

### Polling Order

`tokio::join!` polls futures in the order they appear:
1. Poll future 1
2. Poll future 2
3. Poll future 3
4. ...

If any return `Pending`, the whole join returns `Pending` and waits to be polled again.

## Key Takeaways

1. **Concurrency, Not Parallelism**: `tokio::join!` runs futures concurrently on the same task, not in parallel on different threads

2. **All Must Complete**: Waits for all futures to finish before returning

3. **Order Preserved**: Results returned in argument order, not completion order

4. **Efficiency**: No spawning overhead - runs on the current task

5. **I/O Bound**: Best for I/O-bound operations (network, disk, timers)

6. **Simple Syntax**: Clean, readable way to express concurrent operations

7. **Type Safety**: Compiler ensures correct result types

## Summary

The code demonstrates how `tokio::join!` enables efficient concurrent execution of multiple async operations:

```rust
let (result1, result2) = tokio::join!(task1(), task2());
```

Instead of waiting 150ms for sequential execution, both tasks run concurrently and complete in just 100ms (the time of the longest task). This pattern is fundamental to building high-performance async applications in Rust, allowing you to maximize throughput when dealing with I/O-bound operations like network requests, database queries, or file operations.

The power of `tokio::join!` lies in its simplicity: with a single macro call, you can orchestrate multiple concurrent operations, wait for all of them to complete, and get type-safe results - all while maintaining the clarity and safety that Rust is known for.