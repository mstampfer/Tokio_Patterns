# TCP Client in Rust with Tokio

## Overview

This code creates a TCP (Transmission Control Protocol) client that connects to a server, sends data, and receives a response. It uses Rust's `tokio` library for asynchronous I/O operations.

## Complete Code

```rust
use tokio::net::TcpStream;
use tokio::io::{AsyncReadExt, AsyncWriteExt};

#[tokio::main]
async fn main() -> std::io::Result<()> {
    let mut stream = TcpStream::connect("127.0.0.1:8080").await?;
    
    let message = b"Hello, server!";
    stream.write_all(message).await?;
    
    let mut buffer = [0; 1024];
    let n = stream.read(&mut buffer).await?;
    
    println!("Received: {}", String::from_utf8_lossy(&buffer[..n]));
    
    Ok(())
}
```

## How It Works

### 1. Imports

```rust
use tokio::net::TcpStream;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
```

- **`TcpStream`**: Provides the TCP connection functionality
- **`AsyncReadExt` and `AsyncWriteExt`**: Trait imports that enable async read/write operations on the stream

### 2. Async Runtime Setup

```rust
#[tokio::main]
async fn main() -> std::io::Result<()> {
```

- **`#[tokio::main]`**: A macro that sets up the Tokio async runtime
- **`async fn main()`**: Declares the main function as asynchronous
- **`std::io::Result<()>`**: Returns a Result type for error handling

### 3. Establishing Connection

```rust
let mut stream = TcpStream::connect("127.0.0.1:8080").await?;
```

- **`TcpStream::connect("127.0.0.1:8080")`**: Attempts to connect to a server at:
  - `127.0.0.1`: localhost (the local machine)
  - `8080`: the port number
- **`.await?`**: Waits for the connection to complete and propagates any errors

### 4. Sending Data

```rust
let message = b"Hello, server!";
stream.write_all(message).await?;
```

- **`b"Hello, server!"`**: Creates a byte string literal (array of bytes)
- **`write_all(message)`**: Writes all bytes to the server
- **`.await?`**: Waits for the write operation to complete and handles errors

### 5. Receiving Response

```rust
let mut buffer = [0; 1024];
let n = stream.read(&mut buffer).await?;
```

- **`[0; 1024]`**: Creates a buffer of 1024 bytes initialized to zero
- **`stream.read(&mut buffer)`**: Reads data from the server into the buffer
- **`n`**: Stores the number of bytes actually read

### 6. Displaying Response

```rust
println!("Received: {}", String::from_utf8_lossy(&buffer[..n]));
```

- **`&buffer[..n]`**: Takes only the bytes that were actually read
- **`String::from_utf8_lossy()`**: Converts bytes to a UTF-8 string (replaces invalid UTF-8 with �)
- **`println!`**: Prints the received message to the console

### 7. Return Success

```rust
Ok(())
```

Returns `Ok` to indicate successful execution.

## Key Concepts

### Asynchronous Programming

This code uses async/await to handle I/O operations without blocking. This allows the program to efficiently wait for network operations while potentially handling other tasks.

### Error Handling

The `?` operator is used throughout to propagate errors up the call stack. If any operation fails, the error is returned immediately.

### TCP Communication Flow

1. **Connect**: Client establishes a connection to the server
2. **Send**: Client sends data to the server
3. **Receive**: Client waits for and reads the server's response
4. **Close**: Connection is automatically closed when `stream` goes out of scope

## Requirements

Add this to your `Cargo.toml`:

```toml
[dependencies]
tokio = { version = "1", features = ["full"] }
```

## Running the Code

Before running this client, you need a TCP server listening on `127.0.0.1:8080`. You can test with a simple server or tools like `netcat`:

```bash
# Terminal 1 (start a simple server)
nc -l 8080

# Terminal 2 (run the client)
cargo run
```

The server will receive "Hello, server!" and any response it sends back will be displayed by the client.