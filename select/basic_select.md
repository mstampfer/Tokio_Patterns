# Understanding `tokio::select!` - Waiting for the First Operation

The `select!` macro polls multiple async operations **concurrently** and proceeds with whichever one completes first:

## Complete Code

```rust
use tokio::time::{sleep, Duration};

async fn operation1() -> &'static str {
    sleep(Duration::from_millis(100)).await;
    "Operation 1"
}

async fn operation2() -> &'static str {
    sleep(Duration::from_millis(50)).await;
    "Operation 2"
}

#[tokio::main]
async fn main() {
    tokio::select! {
        result = operation1() => {
            println!("Got: {}", result);
        }
        result = operation2() => {
            println!("Got: {}", result);
        }
    }
}
```

## How `tokio::select!` Works


```rust
tokio::select! {
    result = operation1() => {
        println!("Got: {}", result);
    }
    result = operation2() => {
        println!("Got: {}", result);
    }
}
```

## Step-by-Step Execution

1. **Both futures start concurrently**: `select!` begins polling both `operation1()` and `operation2()` at the same time

2. **The race**: 
   - `operation1()` sleeps for 100ms
   - `operation2()` sleeps for 50ms

3. **First completion wins**: After 50ms, `operation2()` completes first and returns `"Operation 2"`

4. **Winner takes all**: The second branch executes, printing `"Got: Operation 2"`

5. **Loser is cancelled**: `operation1()` is **dropped/cancelled** - it never finishes its 100ms sleep or returns its value

## Key Points

- **Only one branch executes** - whichever future completes first
- **The slower operation is cancelled** - `operation1()` never completes
- **Each branch has its own scope** - that's why using `result` in both branches is fine (they're in separate scopes)
- **Non-deterministic if timings were equal** - if both completed at the same time, either branch could run

This is useful for timeout patterns, racing multiple data sources, or cancelling slow operations.

## Expected Output

```
Got: Operation 2
```

Since `operation2()` completes in 50ms while `operation1()` takes 100ms, `operation2()` will always win the race and its branch will execute.