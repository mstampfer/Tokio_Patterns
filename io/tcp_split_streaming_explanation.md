# TCP Stream Splitting in Tokio

## Overview

This document explains how Tokio allows you to split a TCP stream into separate read and write halves, enabling concurrent read and write operations on the same connection.

## Complete Code

```rust
use tokio::net::TcpStream;
use tokio::io::{AsyncReadExt, AsyncWriteExt};

#[tokio::main]
async fn main() -> std::io::Result<()> {
    let stream = TcpStream::connect("127.0.0.1:8080").await?;
    let (mut reader, mut writer) = stream.into_split();
    
    let write_task = tokio::spawn(async move {
        writer.write_all(b"Hello").await?;
        Ok::<_, std::io::Error>(())
    });
    
    let read_task = tokio::spawn(async move {
        let mut buffer = [0; 1024];
        let n = reader.read(&mut buffer).await?;
        println!("Read {} bytes", n);
        Ok::<_, std::io::Error>(())
    });
    
    write_task.await??;
    read_task.await??;
    
    Ok(())
}
```

## How Stream Splitting Works

### 1. Establishing the Connection

```rust
let stream = TcpStream::connect("127.0.0.1:8080").await?;
```

Creates a single bidirectional TCP connection to the server at `127.0.0.1:8080`.

### 2. Splitting the Stream

```rust
let (mut reader, mut writer) = stream.into_split();
```

The `into_split()` method divides the TCP stream into two independent halves:

- **`OwnedReadHalf` (reader)**: For reading data from the socket
- **`OwnedWriteHalf` (writer)**: For writing data to the socket

**Why `into_split()` instead of `split()`?**

- `split()` returns borrowed references (`&mut`) that cannot be moved into separate tasks
- `into_split()` consumes the original stream and returns **owned** halves
- Owned halves can be moved into different async tasks, enabling true concurrent operations

### 3. Concurrent Operations

Both halves can be used simultaneously in different tasks:

```rust
let write_task = tokio::spawn(async move {
    writer.write_all(b"Hello").await?;
    Ok::<_, std::io::Error>(())
});

let read_task = tokio::spawn(async move {
    let mut buffer = [0; 1024];
    let n = reader.read(&mut buffer).await?;
    println!("Read {} bytes", n);
    Ok::<_, std::io::Error>(())
});
```

- Each half is moved (`async move`) into its own spawned task
- Tasks run concurrently on the Tokio runtime
- The reader can wait for incoming data while the writer sends data

### 4. Awaiting Task Completion

```rust
write_task.await??;
read_task.await??;
```

The double `??` operator handles two levels of errors:

1. **First `?`**: Handles `JoinError` from `tokio::spawn`
   - Occurs if the task panics or is cancelled
2. **Second `?`**: Handles `std::io::Error` from the I/O operations
   - Occurs if reading/writing fails

## Benefits of Stream Splitting

1. **Concurrent I/O**: Read and write operations happen independently without blocking each other
2. **Simplified Logic**: Separate read and write logic into different tasks
3. **Full-Duplex Communication**: True bidirectional communication where both sides can send/receive simultaneously
4. **Task Isolation**: Errors in one task don't directly affect the other

## Under the Hood

Internally, `into_split()` uses `Arc` (Atomic Reference Counting) to share ownership of the underlying socket between the two halves. Each half maintains its own buffer and state, but they coordinate access to the same underlying file descriptor.

## Common Use Cases

- **Chat applications**: Continuously read messages while allowing users to send messages
- **Streaming protocols**: Upload data while downloading responses
- **Bidirectional RPC**: Send requests while processing incoming responses
- **Proxies**: Forward data in both directions simultaneously

## Important Notes

- Both halves must eventually be dropped for the connection to close gracefully
- Closing one half doesn't automatically close the other
- The original `TcpStream` is consumed by `into_split()` and cannot be recovered