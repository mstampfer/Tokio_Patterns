# Understanding Tokio Timeout with I/O Operations

## Overview

This document explains how to add timeouts to asynchronous I/O operations in Rust using `tokio::time::timeout`. The example demonstrates reading from a TCP stream with a 5-second timeout.

## Complete Code

```rust
use tokio::net::TcpStream;
use tokio::io::AsyncReadExt;
use tokio::time::{timeout, Duration};

#[tokio::main]
async fn main() {
    match TcpStream::connect("127.0.0.1:8080").await {
        Ok(mut stream) => {
            let mut buffer = [0; 1024];
            
            let result = timeout(
                Duration::from_secs(5),
                stream.read(&mut buffer)
            ).await;
            
            match result {
                Ok(Ok(n)) => println!("Read {} bytes", n),
                Ok(Err(e)) => println!("I/O error: {}", e),
                Err(_) => println!("Operation timed out"),
            }
        }
        Err(e) => println!("Connection error: {}", e),
    }
}
```

## How It Works

### 1. The Timeout Function

```rust
let result = timeout(
    Duration::from_secs(5),
    stream.read(&mut buffer)
).await;
```

The `timeout` function wraps any async operation and returns early if the operation doesn't complete within the specified duration.

**Parameters:**
- **First argument**: `Duration::from_secs(5)` - The maximum time to wait (5 seconds)
- **Second argument**: `stream.read(&mut buffer)` - The async operation to execute

**Return Type**: `Result<T, Elapsed>` where `T` is the result of the inner operation

### 2. Nested Result Handling

The `timeout` function produces a nested `Result` structure:

```
Result<Result<usize, io::Error>, Elapsed>
   │      │       │        │         │
   │      │       │        │         └─ Timeout error
   │      │       │        └─ I/O error from read operation
   │      │       └─ Number of bytes read (success)
   │      └─ Inner Result from stream.read()
   └─ Outer Result from timeout()
```

### 3. Match Pattern Breakdown

```rust
match result {
    Ok(Ok(n)) => println!("Read {} bytes", n),
    Ok(Err(e)) => println!("I/O error: {}", e),
    Err(_) => println!("Operation timed out"),
}
```

- **`Ok(Ok(n))`**: The timeout did NOT expire AND the read operation succeeded. `n` contains the number of bytes read.
- **`Ok(Err(e))`**: The timeout did NOT expire BUT the read operation failed with an I/O error.
- **`Err(_)`**: The timeout expired before the read operation completed.

## Execution Flow

1. **Connect to server**: Establish TCP connection to `127.0.0.1:8080`
2. **Create buffer**: Allocate a 1024-byte buffer to store incoming data
3. **Wrap with timeout**: The `timeout` function creates a race between:
   - The read operation completing
   - A 5-second timer expiring
4. **Await the result**: The `.await` executes the timeout future
5. **Handle outcome**: Match on the nested Result to determine what happened

## Key Points

### Why `.await` is Required

The `timeout` function returns a `Future` that must be awaited:

```rust
// Wrong - returns a Future, doesn't execute
let result = timeout(Duration::from_secs(5), stream.read(&mut buffer));

// Correct - awaits the Future and gets the Result
let result = timeout(Duration::from_secs(5), stream.read(&mut buffer)).await;
```

### Timeout Behavior

- If the operation completes within 5 seconds, you get `Ok(operation_result)`
- If 5 seconds elapse, the operation is cancelled and you get `Err(Elapsed)`
- The timeout includes ALL time spent in the operation, including any internal delays

### Use Cases

Timeouts are essential for:
- Preventing indefinite hangs on slow or unresponsive servers
- Implementing retry logic with bounded wait times
- Meeting SLA requirements for response times
- Resource management in high-load scenarios

## Common Patterns

### With Error Propagation

```rust
use tokio::time::{timeout, Duration, error::Elapsed};

async fn read_with_timeout(stream: &mut TcpStream) -> Result<usize, Box<dyn std::error::Error>> {
    let mut buffer = [0; 1024];
    let n = timeout(Duration::from_secs(5), stream.read(&mut buffer))
        .await??; // Double ? handles both timeout and I/O errors
    Ok(n)
}
```

### Multiple Operations with Same Timeout

```rust
let result = timeout(Duration::from_secs(10), async {
    stream.write_all(b"GET / HTTP/1.1\r\n\r\n").await?;
    stream.read(&mut buffer).await
}).await;
```

## Dependencies

Add to your `Cargo.toml`:

```toml
[dependencies]
tokio = { version = "1", features = ["full"] }
```

The `full` feature includes networking, I/O, and timing utilities needed for this example.