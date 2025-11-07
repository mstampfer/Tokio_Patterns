# Understanding Biased Selection in `tokio::select!`

## Overview

This code demonstrates how to use **biased selection** in `tokio::select!` to prioritize certain branches over others. By default, `select!` randomly chooses between ready branches to ensure fairness, but with `biased` mode, branches are checked in order, making selection deterministic and allowing you to implement priority-based processing.

## The `biased` Modifier

```rust
tokio::select! {
    biased;
    
    val = rx1.recv() => { ... }
    val = rx2.recv() => { ... }
}
```

The `biased;` keyword at the start of the `select!` block changes the selection behavior:

- **Default (random)**: Branches are polled in a random order to ensure fairness
- **Biased (ordered)**: Branches are checked in the order they appear, top to bottom

## How It Works

### Step 1: Setup Channels and Send Messages

```rust
let (tx1, mut rx1) = mpsc::channel::<i32>(32);
let (tx2, mut rx2) = mpsc::channel::<i32>(32);

tx1.send(1).await.unwrap();
tx2.send(2).await.unwrap();
```

Two channels are created, and messages are sent to both immediately. This means **both channels have ready messages** when `select!` runs.

### Step 2: Drop Senders

```rust
drop(tx1);
drop(tx2);
```

The senders are explicitly dropped to close the channels. This ensures the program doesn't hang waiting for potential future messages after processing.

### Step 3: Biased Selection

```rust
tokio::select! {
    biased;
    
    val = rx1.recv() => {
        if let Some(val) = val {
            println!("Channel 1: {}", val);
        }
    }
    val = rx2.recv() => {
        if let Some(val) = val {
            println!("Channel 2: {}", val);
        }
    }
}
```

**What happens:**

1. The `biased;` modifier is specified
2. Both channels have messages ready
3. `select!` checks branches **in order**:
   - First checks `rx1.recv()` → Ready! Processes this branch
   - Never checks `rx2.recv()` because the first branch was ready
4. Prints `"Channel 1: 1"`
5. Program exits (only one message processed)

## Comparison: Default vs Biased

### Default Behavior (Random Selection)

```rust
tokio::select! {
    val = rx1.recv() => {
        if let Some(val) = val {
            println!("Channel 1: {}", val);
        }
    }
    val = rx2.recv() => {
        if let Some(val) = val {
            println!("Channel 2: {}", val);
        }
    }
}
```

**Output** (non-deterministic):
- Could be `Channel 1: 1` (roughly 50% of the time)
- Could be `Channel 2: 2` (roughly 50% of the time)

The runtime randomly chooses which ready branch to execute to prevent starvation.

### Biased Behavior (Ordered Selection)

```rust
tokio::select! {
    biased;
    
    val = rx1.recv() => {
        if let Some(val) = val {
            println!("Channel 1: {}", val);
        }
    }
    val = rx2.recv() => {
        if let Some(val) = val {
            println!("Channel 2: {}", val);
        }
    }
}
```

**Output** (deterministic):
- Always `Channel 1: 1`

The first ready branch in source order always wins.

## Complete Code

```rust
use tokio::sync::mpsc;

#[tokio::main]
async fn main() {
    let (tx1, mut rx1) = mpsc::channel::<i32>(32);
    let (tx2, mut rx2) = mpsc::channel::<i32>(32);
    
    // Send to both channels immediately
    tx1.send(1).await.unwrap();
    tx2.send(2).await.unwrap();
    
    // Drop senders so receivers know no more messages are coming
    drop(tx1);
    drop(tx2);
    
    // Add biased modifier to make selection deterministic
    tokio::select! {
        biased;
        
        val = rx1.recv() => {
            if let Some(val) = val {
                println!("Channel 1: {}", val);
            }
        }
        val = rx2.recv() => {
            if let Some(val) = val {
                println!("Channel 2: {}", val);
            }
        }
    }
}
```

## Expected Output

```
Channel 1: 1
```

With `biased` mode, Channel 1 is always prioritized when both channels have messages ready.

## Practical Example: Priority-Based Task Processing

Here's a more realistic example showing how biased selection implements priority:

```rust
use tokio::sync::mpsc;
use tokio::time::{sleep, Duration};

#[tokio::main]
async fn main() {
    let (high_priority_tx, mut high_priority_rx) = mpsc::channel::<String>(32);
    let (low_priority_tx, mut low_priority_rx) = mpsc::channel::<String>(32);
    
    // Spawn tasks that generate work at different priorities
    tokio::spawn(async move {
        for i in 0..5 {
            high_priority_tx.send(format!("URGENT: Task {}", i)).await.unwrap();
            sleep(Duration::from_millis(100)).await;
        }
    });
    
    tokio::spawn(async move {
        for i in 0..10 {
            low_priority_tx.send(format!("Normal: Task {}", i)).await.unwrap();
            sleep(Duration::from_millis(50)).await;
        }
    });
    
    let mut high_open = true;
    let mut low_open = true;
    
    loop {
        tokio::select! {
            biased;  // Process high priority first!
            
            msg = high_priority_rx.recv(), if high_open => {
                match msg {
                    Some(task) => {
                        println!("Processing: {}", task);
                        sleep(Duration::from_millis(30)).await;
                    }
                    None => high_open = false,
                }
            }
            
            msg = low_priority_rx.recv(), if low_open => {
                match msg {
                    Some(task) => {
                        println!("Processing: {}", task);
                        sleep(Duration::from_millis(30)).await;
                    }
                    None => low_open = false,
                }
            }
        }
        
        if !high_open && !low_open {
            break;
        }
    }
    
    println!("All tasks completed");
}
```

### Output Pattern:

```
Processing: URGENT: Task 0
Processing: Normal: Task 0
Processing: URGENT: Task 1
Processing: Normal: Task 1
Processing: Normal: Task 2
Processing: URGENT: Task 2
...
```

Notice that whenever a high-priority task is ready, it gets processed first, even if low-priority tasks are also waiting.

## When to Use Biased Selection

### Good Use Cases ✅

1. **Priority queues**: Process high-priority messages before low-priority ones
2. **Shutdown signals**: Check for shutdown requests before processing work
3. **Control messages**: Handle control/admin commands before regular data
4. **Resource management**: Prioritize cleanup or maintenance tasks
5. **Testing**: Make behavior deterministic for easier testing

### Example: Graceful Shutdown

```rust
tokio::select! {
    biased;
    
    _ = shutdown_signal.recv() => {
        println!("Shutting down gracefully...");
        // Cleanup code
        return;
    }
    
    work = work_queue.recv() => {
        // Process work
    }
}
```

The shutdown signal is checked first, ensuring clean shutdown even if work is available.

### When NOT to Use Biased ⚠️

1. **Fair processing**: When all branches should have equal opportunity
2. **Preventing starvation**: If one branch is frequently ready, later branches may never execute
3. **Load balancing**: When you want to distribute work evenly

## The Starvation Problem

**Warning**: Biased selection can cause **starvation** of lower-priority branches!

```rust
tokio::select! {
    biased;
    
    // If this is ALWAYS ready...
    msg = high_rate_channel.recv() => { ... }
    
    // ...this may NEVER execute!
    msg = low_rate_channel.recv() => { ... }
}
```

If the first branch is always or frequently ready, the second branch might never get a chance to execute. This is why the default random selection exists - to prevent starvation.

## Key Differences Summary

| Feature | Default (Random) | Biased (Ordered) |
|---------|-----------------|------------------|
| **Branch selection** | Random order | Top-to-bottom order |
| **Fairness** | All branches have equal chance | Higher branches prioritized |
| **Starvation risk** | Low (prevented by randomization) | High (lower branches may starve) |
| **Determinism** | Non-deterministic | Deterministic |
| **Use case** | General multiplexing | Priority-based processing |

## Best Practices

1. **Use `biased` sparingly**: Only when you genuinely need priority
2. **Document priority reasoning**: Make it clear why certain branches are prioritized
3. **Monitor for starvation**: Ensure lower-priority branches still get processed
4. **Consider alternatives**: Sometimes separate loops or spawn priorities are better
5. **Test both modes**: Verify behavior with and without `biased` during development

## Complete Code (Simple Version)

```rust
use tokio::sync::mpsc;

#[tokio::main]
async fn main() {
    let (tx1, mut rx1) = mpsc::channel::<i32>(32);
    let (tx2, mut rx2) = mpsc::channel::<i32>(32);
    
    // Send to both channels immediately
    tx1.send(1).await.unwrap();
    tx2.send(2).await.unwrap();
    
    // Drop senders so receivers know no more messages are coming
    drop(tx1);
    drop(tx2);
    
    // Add biased modifier to make selection deterministic
    tokio::select! {
        biased;
        
        val = rx1.recv() => {
            if let Some(val) = val {
                println!("Channel 1: {}", val);
            }
        }
        val = rx2.recv() => {
            if let Some(val) = val {
                println!("Channel 2: {}", val);
            }
        }
    }
}
```

## Conclusion

The `biased` modifier in `tokio::select!` is a powerful tool for implementing priority-based async processing. It transforms `select!` from a fair multiplexer into a priority queue, checking branches in order and always choosing the first ready branch. Use it when priority matters, but be aware of the starvation risks for lower-priority branches.