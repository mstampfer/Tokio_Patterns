# Pattern Matching with Enum Messages in Tokio Channels

## Overview

This code demonstrates how to use Rust's pattern matching to handle different types of messages received from a tokio channel. By using an enum to represent different message variants, we can send and receive heterogeneous data through a single channel.

## The Message Enum

```rust
enum Message {
    Text(String),
    Number(i32),
    Quit,
}
```

This enum defines three types of messages:
- **`Text(String)`**: Carries a text message
- **`Number(i32)`**: Carries a numeric value
- **`Quit`**: A signal to quit (no associated data)

## Pattern Matching Strategy

The code uses a two-level approach to handle messages:

1. **`tokio::select!`**: Waits for async operations to complete (in this case, just receiving from the channel)
2. **`match` statement**: Pattern matches on the received message to determine its type and extract its data

```rust
tokio::select! {
    msg = rx.recv() => {
        match msg {
            Some(Message::Text(s)) => {
                println!("Got text: {}", s);
            }
            Some(Message::Number(n)) => {
                println!("Got number: {}", n);
            }
            Some(Message::Quit) => {
                println!("Quitting");
            }
            None => {
                println!("Channel closed");
            }
        }
    }
}
```

## How It Works

### Step 1: Channel Creation
```rust
let (tx, mut rx) = mpsc::channel(32);
```
Creates a multi-producer, single-consumer channel with a buffer size of 32 messages.

### Step 2: Spawning a Sender Task
```rust
tokio::spawn(async move {
    tx.send(Message::Text("Hello".to_string())).await.unwrap();
});
```
A separate async task sends a `Text` message containing "Hello".

### Step 3: Receiving and Pattern Matching
The `select!` block receives the message, and the `match` statement determines which variant was received:

- **`Some(Message::Text(s))`**: Extracts the `String` from the `Text` variant
- **`Some(Message::Number(n))`**: Extracts the `i32` from the `Number` variant
- **`Some(Message::Quit)`**: Matches the `Quit` variant with no data to extract
- **`None`**: Handles the case where the channel is closed and no more messages will arrive

## Key Points

### Why Not Pattern Match in `select!`?

**Invalid approach** (this won't compile):
```rust
tokio::select! {
    Some(Message::Text(s)) = rx.recv() => { ... }
    Some(Message::Number(n)) = rx.recv() => { ... }
}
```

**Why it doesn't work**: 
- `tokio::select!` is for racing **different async operations**, not for discriminating different values from the same operation
- Each branch would create a separate `rx.recv()` call, meaning three independent receive operations competing
- You can only bind the entire result, not pattern match in the branch itself

**Correct approach**:
```rust
tokio::select! {
    msg = rx.recv() => {
        match msg { ... }  // Pattern match inside the branch
    }
}
```

### The `Option` Wrapper

`rx.recv()` returns `Option<Message>`:
- **`Some(Message)`**: A message was successfully received
- **`None`**: The channel was closed (all senders dropped)

This is why we pattern match on `Some(Message::Text(s))` rather than just `Message::Text(s)`.

## Complete Code

```rust
use tokio::sync::mpsc;

enum Message {
    Text(String),
    Number(i32),
    Quit,
}

#[tokio::main]
async fn main() {
    let (tx, mut rx) = mpsc::channel(32);
    
    tokio::spawn(async move {
        tx.send(Message::Text("Hello".to_string())).await.unwrap();
    });
    
    tokio::select! {
        msg = rx.recv() => {
            match msg {
                Some(Message::Text(s)) => {
                    println!("Got text: {}", s);
                }
                Some(Message::Number(n)) => {
                    println!("Got number: {}", n);
                }
                Some(Message::Quit) => {
                    println!("Quitting");
                }
                None => {
                    println!("Channel closed");
                }
            }
        }
    }
}
```

## Expected Output

```
Got text: Hello
```

Since the spawned task sends a `Text` message, the first pattern in the match statement matches and extracts the string "Hello".

## Extended Example: Multiple Messages

Here's a more realistic example that processes multiple messages in a loop:

```rust
use tokio::sync::mpsc;

enum Message {
    Text(String),
    Number(i32),
    Quit,
}

#[tokio::main]
async fn main() {
    let (tx, mut rx) = mpsc::channel(32);
    
    tokio::spawn(async move {
        tx.send(Message::Text("Hello".to_string())).await.unwrap();
        tx.send(Message::Number(42)).await.unwrap();
        tx.send(Message::Text("World".to_string())).await.unwrap();
        tx.send(Message::Quit).await.unwrap();
    });
    
    loop {
        tokio::select! {
            msg = rx.recv() => {
                match msg {
                    Some(Message::Text(s)) => {
                        println!("Got text: {}", s);
                    }
                    Some(Message::Number(n)) => {
                        println!("Got number: {}", n);
                    }
                    Some(Message::Quit) => {
                        println!("Quitting");
                        break;
                    }
                    None => {
                        println!("Channel closed");
                        break;
                    }
                }
            }
        }
    }
}
```

### Output:
```
Got text: Hello
Got number: 42
Got text: World
Quitting
```

## Common Use Cases

This pattern is useful for:

- **Actor model implementations**: Different message types represent different commands
- **Event-driven systems**: Different events trigger different handlers
- **State machines**: Messages drive state transitions
- **Command/query separation**: Distinguish between commands that modify state and queries that read it
- **Protocol implementations**: Handle different protocol message types through a single channel