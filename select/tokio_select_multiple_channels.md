# Using `tokio::select!` in a Loop with Multiple Channels

## Overview

This code demonstrates how to use `tokio::select!` in a loop to concurrently receive messages from multiple channels until all channels are closed. This is a common pattern for multiplexing multiple async streams.

## The Challenge

When working with multiple channels, we need to:

1. **Race between channels**: Receive whichever message arrives first
2. **Handle channels closing at different times**: One channel may close before another
3. **Avoid infinite loops**: Once a channel closes, `recv()` returns `None` immediately, which would cause that branch to trigger repeatedly
4. **Know when to stop**: Exit the loop only when **all** channels are closed

## The Solution: Branch Guards

The key technique is using **branch guards** (`if` conditions) in `tokio::select!` to disable branches for closed channels:

```rust
tokio::select! {
    msg = rx1.recv(), if rx1_open => {
        // Handle channel 1
    }
    msg = rx2.recv(), if rx2_open => {
        // Handle channel 2
    }
}
```

The `if rx1_open` guard means: "Only poll this branch if `rx1_open` is `true`."

## How It Works

### Step 1: Setup Multiple Channels

```rust
let (tx1, mut rx1) = mpsc::channel(32);
let (tx2, mut rx2) = mpsc::channel(32);
```

Two independent channels are created, each with its own sender and receiver.

### Step 2: Spawn Producer Tasks

```rust
tokio::spawn(async move {
    for i in 0..3 {
        tx1.send(format!("Channel 1: {}", i)).await.unwrap();
        sleep(Duration::from_millis(50)).await;
    }
});

tokio::spawn(async move {
    for i in 0..3 {
        tx2.send(format!("Channel 2: {}", i)).await.unwrap();
        sleep(Duration::from_millis(75)).await;
    }
});
```

- **Channel 1**: Sends 3 messages, one every 50ms
- **Channel 2**: Sends 3 messages, one every 75ms

These run concurrently, so messages will be interleaved based on timing.

### Step 3: Track Channel Status

```rust
let mut rx1_open = true;
let mut rx2_open = true;
```

Boolean flags track whether each channel is still open (has not returned `None`).

### Step 4: The Select Loop

```rust
loop {
    tokio::select! {
        msg = rx1.recv(), if rx1_open => {
            match msg {
                Some(msg) => println!("{}", msg),
                None => rx1_open = false,
            }
        }
        msg = rx2.recv(), if rx2_open => {
            match msg {
                Some(msg) => println!("{}", msg),
                None => rx2_open = false,
            }
        }
    }
    
    if !rx1_open && !rx2_open {
        break;
    }
}
```

**What happens in each iteration:**

1. **`select!` races the enabled branches**: Only branches with `true` guards are polled
2. **First ready message wins**: Whichever channel has a message ready is processed
3. **Handle the message**:
   - If `Some(msg)`: Print the message
   - If `None`: Mark that channel as closed by setting its flag to `false`
4. **Check termination condition**: If both channels are closed, exit the loop

### Step 5: Completion

```rust
println!("All messages received");
```

Once both channels are closed and all messages received, the program prints a completion message.

## Execution Timeline

Here's what happens over time:

```
Time    Event
----    -----
0ms     Both tasks spawned, select loop starts
50ms    Channel 1 sends "Channel 1: 0" → printed immediately
75ms    Channel 2 sends "Channel 2: 0" → printed immediately
100ms   Channel 1 sends "Channel 1: 1" → printed immediately
150ms   Channel 1 sends "Channel 1: 2" → printed immediately
        Channel 1 closes (all 3 messages sent)
        rx1_open = false, only rx2 branch active now
150ms   Channel 2 sends "Channel 2: 1" → printed immediately
225ms   Channel 2 sends "Channel 2: 2" → printed immediately
        Channel 2 closes (all 3 messages sent)
        rx2_open = false
        Both channels closed → loop breaks
        "All messages received" printed
```

## Why Branch Guards Are Essential

### Without Guards (Incorrect)

```rust
loop {
    tokio::select! {
        msg = rx1.recv() => {
            match msg {
                Some(msg) => println!("{}", msg),
                None => { /* Channel closed, but what now? */ }
            }
        }
        msg = rx2.recv() => {
            match msg {
                Some(msg) => println!("{}", msg),
                None => { /* Channel closed, but what now? */ }
            }
        }
    }
}
```

**Problem**: Once a channel closes and returns `None`, it will immediately return `None` again on every subsequent call. This means that branch will always be ready, winning every race, and creating an infinite loop.

### With Guards (Correct)

```rust
loop {
    tokio::select! {
        msg = rx1.recv(), if rx1_open => {
            match msg {
                Some(msg) => println!("{}", msg),
                None => rx1_open = false,  // Disable this branch
            }
        }
        msg = rx2.recv(), if rx2_open => {
            match msg {
                Some(msg) => println!("{}", msg),
                None => rx2_open = false,  // Disable this branch
            }
        }
    }
    
    if !rx1_open && !rx2_open {
        break;  // Exit when both closed
    }
}
```

**Solution**: When a channel closes, set its flag to `false`, which disables that branch. The disabled branch is no longer polled, allowing the other channel(s) to continue being processed.

## Complete Code

```rust
use tokio::sync::mpsc;
use tokio::time::{sleep, Duration};

#[tokio::main]
async fn main() {
    let (tx1, mut rx1) = mpsc::channel(32);
    let (tx2, mut rx2) = mpsc::channel(32);
    
    tokio::spawn(async move {
        for i in 0..3 {
            tx1.send(format!("Channel 1: {}", i)).await.unwrap();
            sleep(Duration::from_millis(50)).await;
        }
    });
    
    tokio::spawn(async move {
        for i in 0..3 {
            tx2.send(format!("Channel 2: {}", i)).await.unwrap();
            sleep(Duration::from_millis(75)).await;
        }
    });
    
    let mut rx1_open = true;
    let mut rx2_open = true;
    
    loop {
        tokio::select! {
            msg = rx1.recv(), if rx1_open => {
                match msg {
                    Some(msg) => println!("{}", msg),
                    None => rx1_open = false,
                }
            }
            msg = rx2.recv(), if rx2_open => {
                match msg {
                    Some(msg) => println!("{}", msg),
                    None => rx2_open = false,
                }
            }
        }
        
        if !rx1_open && !rx2_open {
            break;
        }
    }
    
    println!("All messages received");
}
```

## Expected Output

```
Channel 1: 0
Channel 2: 0
Channel 1: 1
Channel 2: 1
Channel 1: 2
Channel 2: 2
All messages received
```

The exact order may vary slightly depending on system timing, but messages from each channel will appear in order (0, 1, 2).

## Key Takeaways

1. **Branch guards (`if` conditions)** allow you to dynamically enable/disable branches in `select!`
2. **Track channel state** with boolean flags to know when channels close
3. **Pattern matching happens inside branches**, not in the binding (use `msg = rx.recv()`, not `Some(msg) = rx.recv()`)
4. **Check all channels are closed** before breaking the loop
5. **`select!` in a loop** is perfect for multiplexing multiple async streams

## Common Use Cases

This pattern is useful for:

- **Multiple data sources**: Aggregate data from multiple APIs or services
- **Event handling**: Process events from different sources as they arrive
- **Load balancing**: Distribute work across multiple workers
- **Monitoring**: Watch multiple channels for system events
- **Protocol multiplexing**: Handle multiple connections or streams simultaneously
- **Fan-in pattern**: Combine outputs from multiple producers into a single consumer

## Alternative: Using `else` Branch

You might have seen code using the `else` branch:

```rust
tokio::select! {
    msg = rx1.recv() => { ... }
    msg = rx2.recv() => { ... }
    else => {
        break;  // All branches would block
    }
}
```

**However, this doesn't work reliably** for our use case because:
- The `else` branch only runs when **all** branches would block (none are ready)
- A closed channel returns `None` immediately (doesn't block)
- So `else` won't trigger when channels are closed

The branch guard approach is more explicit and reliable for handling channel closures.