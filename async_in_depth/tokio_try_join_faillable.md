# Handling Multiple Fallible Futures with tokio::try_join!

## Complete Code

```rust
use tokio::time::{sleep, Duration};

async fn fallible_task1() -> Result<i32, &'static str> {
    sleep(Duration::from_millis(50)).await;
    Ok(1)
}

async fn fallible_task2() -> Result<i32, &'static str> {
    sleep(Duration::from_millis(30)).await;
    Ok(2)
}

#[tokio::main]
async fn main() -> Result<(), &'static str> {
    let (result1, result2) = tokio::try_join!(
        fallible_task1(),
        fallible_task2()
    )?;
    
    println!("Results: {} and {}", result1, result2);
    Ok(())
}
```

## Output (Success Case)

```
Results: 1 and 2
```

## Understanding tokio::try_join!

### What is tokio::try_join!?

`tokio::try_join!` is a variant of `tokio::join!` designed specifically for futures that return `Result`:

```rust
tokio::join!(fut1, fut2)       // For any futures
tokio::try_join!(fut1, fut2)   // For futures returning Result
```

**Key characteristics:**
- Runs all futures **concurrently** (like `join!`)
- Each future must return a `Result<T, E>`
- **Fails fast**: Returns immediately when any future returns `Err`
- Returns `Result<(T1, T2, ...), E>` where the success tuple contains all unwrapped values
- All errors must be the same type `E`

### The Critical Difference: Fail-Fast Behavior

**tokio::join! (non-failing):**
```rust
let (r1, r2) = tokio::join!(task1(), task2());
// r1: Result<i32, &str>
// r2: Result<i32, &str>
// Both always complete, even if one fails
```

**tokio::try_join! (fail-fast):**
```rust
let (v1, v2) = tokio::try_join!(task1(), task2())?;
// Returns: Result<(i32, i32), &str>
// v1: i32 (unwrapped)
// v2: i32 (unwrapped)
// Stops immediately if any task returns Err
```

## Breaking Down the Code

### The Fallible Task Functions

```rust
async fn fallible_task1() -> Result<i32, &'static str> {
    sleep(Duration::from_millis(50)).await;
    Ok(1)
}
```

**Return type:** `Result<i32, &'static str>`
- **Success case:** `Ok(1)` - returns the value `1`
- **Error case:** Could return `Err("some error message")`

```rust
async fn fallible_task2() -> Result<i32, &'static str> {
    sleep(Duration::from_millis(30)).await;
    Ok(2)
}
```

**Return type:** `Result<i32, &'static str>`
- **Success case:** `Ok(2)` - returns the value `2`
- **Error case:** Could return `Err("some error message")`

**Important:** Both tasks have the same error type (`&'static str`) - this is required for `try_join!`

### The try_join! Call

```rust
let (result1, result2) = tokio::try_join!(
    fallible_task1(),
    fallible_task2()
)?;
```

**What happens:**

1. **Creates futures:** Both `fallible_task1()` and `fallible_task2()` are called
2. **Runs concurrently:** Both start executing at the same time
3. **Returns Result:** The macro returns `Result<(i32, i32), &'static str>`
4. **Unwraps with ?:** The `?` operator unwraps the `Ok` or propagates the `Err`
5. **Destructures:** `(result1, result2)` are the unwrapped values

**Type breakdown:**
```rust
tokio::try_join!(task1(), task2())  
// Returns: Result<(i32, i32), &'static str>

tokio::try_join!(task1(), task2())?  
// Returns: (i32, i32) if Ok, or propagates Err

let (result1, result2) = ...
// result1: i32
// result2: i32
```

### The Question Mark Operator

```rust
)?;  // This ? is critical!
```

**Why it's needed:**
- `try_join!` returns a `Result`
- Without `?`, you'd have a `Result` where you expect a tuple
- The `?` operator:
  - If `Ok((v1, v2))`: Unwraps to `(v1, v2)`
  - If `Err(e)`: Returns early from `main()` with `Err(e)`

## Execution Flow

### Success Case (Both Tasks Succeed)

```
Time (ms)    task1 (50ms)              task2 (30ms)
──────────────────────────────────────────────────────────
0            ▼ Start                   ▼ Start
             │ Sleeping...             │ Sleeping...
             │                         │
30           │                         ✓ Returns Ok(2)
             │                         │ (waits for task1)
             │                         │
50           ✓ Returns Ok(1)           │
             │                         │
             └─────────────────────────┘
             try_join! returns Ok((1, 2))
             ? unwraps to (1, 2)
             Prints: "Results: 1 and 2"
```

**Step by step:**

1. **t=0ms:** Both tasks start sleeping
2. **t=30ms:** `task2` completes with `Ok(2)`
3. **t=50ms:** `task1` completes with `Ok(1)`
4. **Both succeeded:** `try_join!` returns `Ok((1, 2))`
5. **`?` unwraps:** Extracts `(1, 2)` from the `Result`
6. **Assignment:** `result1 = 1`, `result2 = 2`
7. **Print:** Outputs "Results: 1 and 2"

### Failure Case (One Task Fails)

Let's modify `task2` to fail:

```rust
async fn fallible_task2() -> Result<i32, &'static str> {
    sleep(Duration::from_millis(30)).await;
    Err("Task 2 failed!")  // Returns an error
}
```

**Execution timeline:**

```
Time (ms)    task1 (50ms)              task2 (30ms)
──────────────────────────────────────────────────────────
0            ▼ Start                   ▼ Start
             │ Sleeping...             │ Sleeping...
             │                         │
30           │                         ✗ Returns Err("Task 2 failed!")
             │                         
             ✋ Task1 may be cancelled
             
             try_join! returns Err("Task 2 failed!") immediately
             ? propagates error
             main() returns Err("Task 2 failed!")
             Program exits with error
```

**Step by step:**

1. **t=0ms:** Both tasks start sleeping
2. **t=30ms:** `task2` completes with `Err("Task 2 failed!")`
3. **Immediate return:** `try_join!` returns `Err("Task 2 failed!")` right away
4. **Cancel task1:** Task1 is dropped (may not complete)
5. **`?` propagates:** Returns the error from `main()`
6. **No print:** The `println!` never executes

## Comparing tokio::join! vs tokio::try_join!

### Using tokio::join! (Always Waits for All)

```rust
let (result1, result2) = tokio::join!(
    fallible_task1(),
    fallible_task2()
);

// result1: Result<i32, &str>
// result2: Result<i32, &str>

match (result1, result2) {
    (Ok(v1), Ok(v2)) => println!("Success: {}, {}", v1, v2),
    (Err(e), _) => println!("Task 1 failed: {}", e),
    (_, Err(e)) => println!("Task 2 failed: {}", e),
}
```

**Characteristics:**
- ✅ Always waits for all tasks to complete
- ✅ Gets all results (success or failure)
- ❌ More verbose error handling
- ❌ Can't use `?` operator directly

### Using tokio::try_join! (Fail-Fast)

```rust
let (result1, result2) = tokio::try_join!(
    fallible_task1(),
    fallible_task2()
)?;

println!("Success: {}, {}", result1, result2);
```

**Characteristics:**
- ✅ Fails fast - returns immediately on first error
- ✅ Concise - use `?` operator
- ✅ Results are already unwrapped
- ❌ Doesn't wait for other tasks if one fails
- ❌ Can only get first error

## Error Type Requirements

### All Errors Must Be the Same Type

```rust
// ✅ WORKS - same error type
async fn task1() -> Result<i32, String> { Ok(1) }
async fn task2() -> Result<i32, String> { Ok(2) }

tokio::try_join!(task1(), task2())?;  // OK
```

```rust
// ❌ DOESN'T WORK - different error types
async fn task1() -> Result<i32, String> { Ok(1) }
async fn task2() -> Result<i32, &'static str> { Ok(2) }

tokio::try_join!(task1(), task2())?;  // Error: mismatched types
```

### Converting Error Types

If you have different error types, convert them:

```rust
async fn task1() -> Result<i32, std::io::Error> { Ok(1) }
async fn task2() -> Result<i32, &'static str> { Ok(2) }

// Convert both to String:
tokio::try_join!(
    async { task1().await.map_err(|e| e.to_string()) },
    async { task2().await.map_err(|e| e.to_string()) }
)?;
```

Or use a common error type:

```rust
#[derive(Debug)]
enum AppError {
    IoError(std::io::Error),
    StringError(&'static str),
}

async fn task1() -> Result<i32, AppError> { Ok(1) }
async fn task2() -> Result<i32, AppError> { Ok(2) }

tokio::try_join!(task1(), task2())?;  // Works!
```

## Practical Examples

### Example 1: Multiple API Calls

```rust
use tokio::time::{sleep, Duration};

async fn fetch_user(id: u32) -> Result<String, &'static str> {
    sleep(Duration::from_millis(100)).await;
    if id == 0 {
        Err("Invalid user ID")
    } else {
        Ok(format!("User {}", id))
    }
}

async fn fetch_profile(id: u32) -> Result<String, &'static str> {
    sleep(Duration::from_millis(80)).await;
    Ok(format!("Profile for user {}", id))
}

async fn fetch_settings(id: u32) -> Result<String, &'static str> {
    sleep(Duration::from_millis(60)).await;
    Ok(format!("Settings for user {}", id))
}

#[tokio::main]
async fn main() -> Result<(), &'static str> {
    let user_id = 42;
    
    let (user, profile, settings) = tokio::try_join!(
        fetch_user(user_id),
        fetch_profile(user_id),
        fetch_settings(user_id)
    )?;
    
    println!("User: {}", user);
    println!("Profile: {}", profile);
    println!("Settings: {}", settings);
    
    Ok(())
}
```

**If all succeed:**
```
User: User 42
Profile: Profile for user 42
Settings: Settings for user 42
```

**If one fails (e.g., user_id = 0):**
- Returns immediately with error "Invalid user ID"
- Other tasks may be cancelled
- No output printed

### Example 2: Database Operations

```rust
async fn insert_user(name: &str) -> Result<u32, &'static str> {
    sleep(Duration::from_millis(50)).await;
    if name.is_empty() {
        Err("Name cannot be empty")
    } else {
        Ok(1) // User ID
    }
}

async fn insert_profile(user_id: u32) -> Result<(), &'static str> {
    sleep(Duration::from_millis(40)).await;
    Ok(())
}

async fn send_welcome_email(user_id: u32) -> Result<(), &'static str> {
    sleep(Duration::from_millis(30)).await;
    Ok(())
}

#[tokio::main]
async fn main() -> Result<(), &'static str> {
    let name = "Alice";
    
    // All must succeed or transaction rolls back
    let (user_id, _, _) = tokio::try_join!(
        insert_user(name),
        insert_profile(1),
        send_welcome_email(1)
    )?;
    
    println!("User created with ID: {}", user_id);
    Ok(())
}
```

### Example 3: File Operations

```rust
async fn read_config() -> Result<String, &'static str> {
    sleep(Duration::from_millis(50)).await;
    Ok("config_data".to_string())
}

async fn read_schema() -> Result<String, &'static str> {
    sleep(Duration::from_millis(60)).await;
    Ok("schema_data".to_string())
}

async fn read_secrets() -> Result<String, &'static str> {
    sleep(Duration::from_millis(40)).await;
    // Simulate a failure
    Err("Secrets file not found")
}

#[tokio::main]
async fn main() -> Result<(), &'static str> {
    match tokio::try_join!(
        read_config(),
        read_schema(),
        read_secrets()
    ) {
        Ok((config, schema, secrets)) => {
            println!("All files loaded successfully");
            println!("Config: {} bytes", config.len());
            println!("Schema: {} bytes", schema.len());
            println!("Secrets: {} bytes", secrets.len());
            Ok(())
        }
        Err(e) => {
            eprintln!("Failed to load files: {}", e);
            Err(e)
        }
    }
}
```

**Output:**
```
Failed to load files: Secrets file not found
```

## Advanced Patterns

### Pattern 1: With Error Context

```rust
tokio::try_join!(
    async {
        fallible_task1()
            .await
            .map_err(|e| format!("Task1 failed: {}", e))
    },
    async {
        fallible_task2()
            .await
            .map_err(|e| format!("Task2 failed: {}", e))
    }
)?;
```

### Pattern 2: With Logging

```rust
let (r1, r2) = tokio::try_join!(
    async {
        println!("Starting task 1");
        let result = fallible_task1().await;
        println!("Task 1 completed: {:?}", result);
        result
    },
    async {
        println!("Starting task 2");
        let result = fallible_task2().await;
        println!("Task 2 completed: {:?}", result);
        result
    }
)?;
```

### Pattern 3: With Timeout

```rust
use tokio::time::timeout;

tokio::try_join!(
    async {
        timeout(Duration::from_secs(5), fallible_task1())
            .await
            .map_err(|_| "Task 1 timeout")?
    },
    async {
        timeout(Duration::from_secs(5), fallible_task2())
            .await
            .map_err(|_| "Task 2 timeout")?
    }
)?;
```

### Pattern 4: Collecting Multiple Results

```rust
// For a fixed number (use try_join!):
let (r1, r2, r3, r4) = tokio::try_join!(
    task1(),
    task2(),
    task3(),
    task4()
)?;

// For a dynamic number (use try_join_all):
use futures::future::try_join_all;

let tasks = vec![
    task1(),
    task2(),
    task3(),
];

let results = try_join_all(tasks).await?;
```

## Error Handling Strategies

### Strategy 1: Propagate Errors (Our Example)

```rust
#[tokio::main]
async fn main() -> Result<(), &'static str> {
    let (r1, r2) = tokio::try_join!(task1(), task2())?;
    println!("Success: {}, {}", r1, r2);
    Ok(())
}
```

**Pros:**
- ✅ Simple and concise
- ✅ Uses `?` operator
- ✅ Idiomatic Rust

**Cons:**
- ❌ Less control over error handling
- ❌ Exits immediately on error

### Strategy 2: Match on Result

```rust
#[tokio::main]
async fn main() {
    match tokio::try_join!(task1(), task2()) {
        Ok((r1, r2)) => {
            println!("Success: {}, {}", r1, r2);
        }
        Err(e) => {
            eprintln!("Error occurred: {}", e);
            // Can do recovery here
        }
    }
}
```

**Pros:**
- ✅ Full control over error handling
- ✅ Can recover from errors
- ✅ Can provide fallback values

**Cons:**
- ❌ More verbose
- ❌ Can't use `?` operator

### Strategy 3: Fallback to join! with Manual Handling

```rust
#[tokio::main]
async fn main() {
    let (r1, r2) = tokio::join!(task1(), task2());
    
    match (r1, r2) {
        (Ok(v1), Ok(v2)) => {
            println!("Both succeeded: {}, {}", v1, v2);
        }
        (Ok(v1), Err(e2)) => {
            println!("Task 1 succeeded: {}, Task 2 failed: {}", v1, e2);
        }
        (Err(e1), Ok(v2)) => {
            println!("Task 1 failed: {}, Task 2 succeeded: {}", e1, v2);
        }
        (Err(e1), Err(e2)) => {
            println!("Both failed: {}, {}", e1, e2);
        }
    }
}
```

**Pros:**
- ✅ Gets all results, even if some fail
- ✅ Can handle partial success
- ✅ Most flexible

**Cons:**
- ❌ Very verbose
- ❌ More complex logic

## When to Use tokio::try_join!

### ✅ Use tokio::try_join! When:

1. **All operations must succeed**
   - Can't proceed if any fails
   - Example: Multi-step transaction

2. **Want fail-fast behavior**
   - Don't waste time if something fails early
   - Example: Loading critical configuration files

3. **Same error type across tasks**
   - All return `Result<T, E>` with same `E`

4. **Independent operations**
   - Tasks don't depend on each other
   - Can run concurrently

5. **Error propagation is acceptable**
   - Okay to bubble error up to caller
   - Using `?` operator

### ❌ Don't Use tokio::try_join! When:

1. **Partial success is acceptable**
   - Some tasks can fail without stopping others
   - Use `tokio::join!` instead

2. **Need all error information**
   - Want to know which tasks failed and why
   - Use `tokio::join!` and inspect all results

3. **Different error types**
   - Tasks return different `Result` types
   - Convert errors first or use `join!`

4. **Tasks depend on each other**
   - One task needs result of another
   - Use sequential `.await` instead

5. **Want to continue on error**
   - Need custom error recovery
   - Use `join!` with manual error handling

## Performance Considerations

### Concurrent Execution

Like `tokio::join!`, `try_join!` runs tasks concurrently:

```rust
// Sequential (slow):
let r1 = task1().await?;  // 50ms
let r2 = task2().await?;  // 30ms
// Total: 80ms

// Concurrent (fast):
let (r1, r2) = tokio::try_join!(task1(), task2())?;
// Total: 50ms (the longest task)
```

### Early Cancellation

When a task fails:

```rust
// task1: 100ms
// task2: 50ms (fails)

tokio::try_join!(task1(), task2())?;

// Timeline:
// t=50ms: task2 fails
// Result: Immediately return error
// task1 may be cancelled (doesn't waste 50ms more)
```

This is more efficient than waiting for all tasks to complete.

## Comparison Table

| Feature | tokio::join! | tokio::try_join! | Sequential |
|---------|-------------|------------------|-----------|
| **Execution** | Concurrent | Concurrent | Sequential |
| **Return type** | Tuple of Results | Result of Tuple | Individual Results |
| **Waits for all** | ✅ Always | ❌ No (fails fast) | ✅ Always |
| **On error** | Gets all errors | Returns first error | Stops at first error |
| **Use `?`** | ❌ Can't directly | ✅ Yes | ✅ Yes |
| **Verbosity** | Medium | Low | Low |
| **Best for** | Partial success OK | All must succeed | Dependent tasks |

## Key Takeaways

1. **Fail-Fast**: `try_join!` returns immediately when any future returns `Err`

2. **Requires `?`**: The macro returns a `Result` that needs to be unwrapped

3. **Same Error Type**: All futures must have the same error type `E`

4. **Unwrapped Values**: On success, you get the unwrapped values, not `Result`s

5. **Concurrent Execution**: Tasks still run concurrently like `join!`

6. **Error Propagation**: Natural integration with `?` operator for clean code

7. **All-or-Nothing**: Perfect for operations where partial success isn't useful

## Summary

`tokio::try_join!` is a powerful tool for running multiple fallible operations concurrently with fail-fast semantics. It combines the concurrency benefits of `tokio::join!` with the error handling convenience of the `?` operator:

```rust
let (result1, result2) = tokio::try_join!(
    fallible_task1(),
    fallible_task2()
)?;
```

This pattern is ideal when you need all operations to succeed and want to avoid wasting time if any fails. It's commonly used for:
- Loading multiple configuration files
- Making parallel API calls where all are required
- Multi-step transactions that must be atomic
- Concurrent validation checks

The fail-fast behavior ensures efficient resource usage and clean error handling, making it a go-to tool for building robust async applications in Rust.