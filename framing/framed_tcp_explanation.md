# Framed TCP Messaging with SinkExt

## Overview

This code demonstrates how to use `SinkExt` from the `futures` crate to send framed messages over a TCP stream in Rust. It combines Tokio's async TCP networking with message framing to handle line-based protocol communication.

## Complete Code

```rust
use tokio::net::TcpStream;
use tokio_util::codec::{Framed, LinesCodec};
use futures::SinkExt;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let stream = TcpStream::connect("127.0.0.1:8080").await?;
    let mut framed = Framed::new(stream, LinesCodec::new());
    
    framed.send("Hello, framed world!").await?;
    
    Ok(())
}
```

## How It Works

### 1. **TcpStream Connection**

```rust
let stream = TcpStream::connect("127.0.0.1:8080").await?;
```

Establishes an asynchronous TCP connection to `127.0.0.1:8080`. The `await` keyword makes this non-blocking, allowing other tasks to run while waiting for the connection.

### 2. **Framing with LinesCodec**

```rust
let mut framed = Framed::new(stream, LinesCodec::new());
```

The `Framed` wrapper transforms the raw TCP stream into a higher-level abstraction:

- **LinesCodec**: A codec that frames messages using newline delimiters (`\n`)
- **Framed**: Combines the stream with the codec, providing both `Sink` and `Stream` traits
- This handles the low-level details of buffering and message boundaries

### 3. **SinkExt Trait Usage**

```rust
framed.send("Hello, framed world!").await?;
```

This is where `SinkExt` comes into play:

- **SinkExt** is an extension trait from the `futures` crate that adds convenient methods to any type implementing the `Sink` trait
- The `send()` method is provided by `SinkExt` and does the following:
  1. Takes ownership of the value to send
  2. Encodes it using the `LinesCodec` (adds a newline character)
  3. Writes the framed data to the TCP stream
  4. Flushes the stream to ensure the data is sent
  5. Returns a `Future` that resolves when the operation completes

- **Why `.await?`**: The `send` method is asynchronous and returns a `Result`, so we must await it and handle potential errors with the `?` operator

## Key Concepts

### Sink Trait

The `Sink` trait represents a destination for asynchronous values. It's similar to an async version of an iterator, but in reverse—you push values into it rather than pulling values from it.

### SinkExt Methods

`SinkExt` provides several useful methods beyond `send()`:

- `send()`: Sends a single item and flushes
- `send_all()`: Sends all items from a stream
- `feed()`: Sends an item without flushing (more efficient for batching)
- `flush()`: Explicitly flushes the sink

### Message Framing

Without framing, TCP is just a byte stream with no built-in message boundaries. The `LinesCodec`:

- **Encoding**: Automatically appends `\n` to each message sent
- **Decoding**: Splits incoming data on `\n` to reconstruct messages
- This ensures the receiver knows where one message ends and another begins

## Benefits of This Approach

1. **Abstraction**: You work with logical messages instead of raw bytes
2. **Automatic Buffering**: The codec handles buffering internally
3. **Error Handling**: Encoding/decoding errors are propagated properly
4. **Composability**: Easy to swap different codecs (JSON, length-prefixed, etc.)
5. **Async-Friendly**: Integrates seamlessly with Tokio's async runtime

## Example Use Case

This pattern is commonly used for:

- Chat applications (line-based messages)
- Simple protocol implementations (Redis, Memcached)
- Log streaming
- Any application needing message-based communication over TCP

## Dependencies Required

```toml
[dependencies]
tokio = { version = "1", features = ["full"] }
tokio-util = { version = "0.7", features = ["codec"] }
futures = "0.3"
```