# Tokio Task Cancellation Explanation

This code demonstrates **task cancellation** in Tokio - how to stop a running asynchronous task before it completes naturally. Here's what it does:

## Code Example

```rust
use tokio::time::{sleep, Duration};
#[tokio::main]
async fn main() {
    let handle = tokio::spawn(async {
        loop {
            println!("Working...");
            sleep(Duration::from_millis(100)).await;
        }
    });
    
    sleep(Duration::from_millis(250)).await;
    
    // Cancel the task
    handle.abort();
    
    // Try to await and handle the abort
    match handle.await {
        Ok(_) => println!("Task completed"),
        Err(e) if e.is_cancelled() => println!("Task was cancelled"),
        Err(e) => println!("Task failed: {:?}", e),
    }
}
```

## Overall Purpose

The code spawns a task that runs indefinitely in a loop, lets it run for a short time, then forcefully cancels it and handles the cancellation gracefully.

## Breaking Down the Code

1. **`tokio::spawn(async { loop { ... } })`**
   - Spawns a task that runs an infinite loop
   - This task would run forever if not stopped externally

2. **Inside the loop:**
   - `println!("Working...");` - Prints a message each iteration
   - `sleep(Duration::from_millis(100)).await;` - Pauses for 100ms between iterations
   - This creates a task that prints "Working..." approximately every 100 milliseconds

3. **`sleep(Duration::from_millis(250)).await;`** (in main)
   - The main task sleeps for 250ms
   - During this time, the spawned task runs and prints "Working..." about 2-3 times

4. **`handle.abort();`**
   - Immediately cancels the spawned task
   - This is a forceful cancellation - the task doesn't get a chance to clean up
   - The task will stop at the next `.await` point

5. **`match handle.await { ... }`**
   - Attempts to await the task handle
   - Since the task was aborted, `await` will return an error
   - The match expression handles three cases:
     - `Ok(_)` - Task completed normally (won't happen here)
     - `Err(e) if e.is_cancelled()` - Task was cancelled (this will match)
     - `Err(e)` - Task panicked or failed for another reason

## Execution Timeline

```
Time 0ms:    Spawn infinite task, it starts printing "Working..."
Time 100ms:  Task prints "Working..." (1st time)
Time 200ms:  Task prints "Working..." (2nd time)
Time 250ms:  Main task wakes up and calls abort()
             Task is cancelled
Time 250ms+: Match statement detects cancellation
             Prints "Task was cancelled"
```

## Expected Output

```
Working...
Working...
Working...
Task was cancelled
```

(You might see 2 or 3 "Working..." messages depending on exact timing)

## Key Concepts

- **Cooperative cancellation:** The task only stops at `.await` points, not immediately
- **`abort()` vs graceful shutdown:** This is a hard stop - no cleanup occurs
- **Error handling:** Checking `is_cancelled()` distinguishes abortion from other failures
- **Infinite tasks:** Common pattern for background services that need external cancellation

## Use Cases

This pattern is useful for:
- Background monitoring tasks that need to be stopped
- Worker tasks that should run until explicitly cancelled
- Timeout implementations where a task must be stopped after a deadline
- Gracefully shutting down services

