# Bidirectional TCP Stream Proxy in Rust

## Overview

This code implements a simple TCP proxy that bidirectionally copies data between two TCP streams using Tokio's async runtime. It connects to two separate TCP endpoints and transfers data in both directions simultaneously.

## Complete Code

```rust
use tokio::net::TcpStream;
use tokio::io;

async fn proxy(mut stream1: TcpStream, mut stream2: TcpStream) -> io::Result<()> {
    // Copy data bidirectionally between the two streams
    let (bytes_to_stream2, bytes_to_stream1) = 
        io::copy_bidirectional(&mut stream1, &mut stream2).await?;
    
    println!("Transferred {} bytes to stream2", bytes_to_stream2);
    println!("Transferred {} bytes to stream1", bytes_to_stream1);
    
    Ok(())
}

#[tokio::main]
async fn main() -> io::Result<()> {
    let stream1 = TcpStream::connect("127.0.0.1:8080").await?;
    let stream2 = TcpStream::connect("127.0.0.1:8081").await?;
    
    proxy(stream1, stream2).await?;
    
    Ok(())
}
```

## How Bidirectional Copying Works

### 1. Connection Establishment

```rust
let stream1 = TcpStream::connect("127.0.0.1:8080").await?;
let stream2 = TcpStream::connect("127.0.0.1:8081").await?;
```

The code establishes two separate TCP connections:
- **stream1**: Connects to localhost port 8080
- **stream2**: Connects to localhost port 8081

Both connections are established asynchronously using `.await`.

### 2. The `proxy` Function

```rust
async fn proxy(mut stream1: TcpStream, mut stream2: TcpStream) -> io::Result<()>
```

This function takes ownership of both streams (marked as `mut` for mutability) and handles the bidirectional data transfer.

### 3. Bidirectional Copy Operation

```rust
let (bytes_to_stream2, bytes_to_stream1) = 
    io::copy_bidirectional(&mut stream1, &mut stream2).await?;
```

This is the core of the implementation. The `io::copy_bidirectional` function:

**What it does:**
- Simultaneously copies data from stream1 to stream2 AND from stream2 to stream1
- Runs both copy operations concurrently using async tasks internally
- Continues until both streams reach EOF (end of file) or an error occurs

**How it works internally:**
1. Spawns two concurrent tasks:
   - Task A: Reads from stream1 → Writes to stream2
   - Task B: Reads from stream2 → Writes to stream1
2. Uses efficient buffered I/O to minimize syscalls
3. Automatically handles backpressure (when one side is faster than the other)
4. Returns when both directions complete or either encounters an error

**Return value:**
- A tuple `(u64, u64)` containing:
  - `bytes_to_stream2`: Total bytes copied from stream1 to stream2
  - `bytes_to_stream1`: Total bytes copied from stream2 to stream1

### 4. Error Handling

The `.await?` operator serves two purposes:
- `.await`: Waits for the async operation to complete
- `?`: Propagates any I/O errors up the call stack

If either stream encounters an error (connection closed, network issue, etc.), the error is returned immediately.

### 5. Async Runtime

```rust
#[tokio::main]
```

This attribute macro sets up the Tokio async runtime, which:
- Creates a thread pool for async task execution
- Manages the event loop for I/O operations
- Enables the use of `.await` syntax in the main function

## Data Flow Diagram

```
┌─────────────┐                    ┌─────────────┐
│   Port      │                    │   Port      │
│   8080      │                    │   8081      │
└──────┬──────┘                    └──────┬──────┘
       │                                  │
       │ stream1                 stream2  │
       │                                  │
       └──────────┐          ┌───────────┘
                  │          │
                  ▼          ▼
           ┌──────────────────────┐
           │  copy_bidirectional  │
           │                      │
           │  ┌────────────────┐  │
           │  │ Task A: 1→2    │  │
           │  └────────────────┘  │
           │  ┌────────────────┐  │
           │  │ Task B: 2→1    │  │
           │  └────────────────┘  │
           └──────────────────────┘
```

## Use Cases

This pattern is commonly used for:
- **TCP proxies**: Forwarding traffic between two endpoints
- **Port forwarding**: Redirecting connections from one port to another
- **Protocol bridging**: Converting between different network protocols
- **Load balancing**: Distributing connections across multiple backends
- **Debugging**: Intercepting and logging network traffic

## Key Features

1. **Concurrent bidirectional transfer**: Both directions operate simultaneously without blocking each other
2. **Efficient I/O**: Uses Tokio's async I/O for non-blocking operations
3. **Automatic cleanup**: Streams are automatically closed when the function returns
4. **Error propagation**: Errors from either stream are properly handled and returned
5. **Transparent data transfer**: Data passes through without modification

## Potential Improvements

For production use, consider adding:
- Error logging and recovery
- Connection timeouts
- Graceful shutdown handling
- Metrics and monitoring
- Support for multiple concurrent connections
- Configuration for buffer sizes