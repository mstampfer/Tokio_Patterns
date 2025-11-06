# Length-Delimited Framing in Rust with `LengthDelimitedCodec`

## Overview

The `LengthDelimitedCodec` from the `tokio-util` crate provides automatic message framing for TCP streams by prefixing each message with its length. This solves the fundamental problem of determining where one message ends and another begins in a continuous byte stream.

## The Problem It Solves

TCP is a stream-oriented protocol, meaning data flows as a continuous sequence of bytes with no inherent message boundaries. Without framing:

- You can't tell where one message ends and another begins
- Messages can arrive fragmented or concatenated
- You need custom logic to parse the stream

## How `LengthDelimitedCodec` Works

### Message Format

Each message is encoded with this structure:

```
[Length (4 bytes)] [Message Data (N bytes)]
```

1. **Length Prefix**: A 4-byte (32-bit) unsigned integer indicating the length of the following data
2. **Message Data**: The actual message payload

### Encoding (Sending)

When you call `framed.send(message).await?`:

1. The codec calculates the byte length of your message
2. It prepends a 4-byte length prefix (big-endian by default)
3. It writes both the prefix and message to the TCP stream
4. The receiver can read the prefix first to know how many bytes to expect

### Decoding (Receiving)

When you call `framed.next().await`:

1. The codec reads 4 bytes to get the message length
2. It reads exactly that many bytes from the stream
3. It returns the message data (without the length prefix)
4. Handles partial reads automatically - if data arrives in chunks, it buffers until complete

## Complete Code Example

```rust
use tokio::net::TcpStream;
use tokio_util::codec::{Framed, LengthDelimitedCodec};
use futures::{SinkExt, StreamExt};
use bytes::Bytes;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Connect to a TCP server
    let stream = TcpStream::connect("127.0.0.1:8080").await?;
    
    // Wrap the stream with LengthDelimitedCodec
    // This creates a "framed" stream that handles length prefixing automatically
    let mut framed = Framed::new(stream, LengthDelimitedCodec::new());
    
    // Send a message
    // The codec automatically prepends the length before sending
    let message = Bytes::from("Hello with length prefix");
    framed.send(message).await?;
    
    // Receive a message
    // The codec automatically reads the length prefix and extracts the message
    if let Some(result) = framed.next().await {
        let frame = result?;
        println!("Received: {:?}", frame);
    }
    
    Ok(())
}
```

## Dependencies (Cargo.toml)

```toml
[dependencies]
tokio = { version = "1", features = ["full"] }
tokio-util = { version = "0.7", features = ["codec"] }
futures = "0.3"
bytes = "1"
```

## Key Components

### `Framed<T, U>`

- A wrapper that combines a transport (like `TcpStream`) with a codec
- Implements both `Sink` (for sending) and `Stream` (for receiving)
- Handles the low-level details of encoding/decoding

### `LengthDelimitedCodec`

- Implements the codec trait for length-prefix framing
- Default configuration uses 4-byte big-endian length prefix
- Automatically handles buffering and partial reads

### `SinkExt` and `StreamExt`

- Trait extensions providing `.send()` and `.next()` methods
- Enable ergonomic async/await syntax for sending and receiving

## Wire Format Example

If you send the message `"Hello"` (5 bytes):

```
Wire bytes: [0x00, 0x00, 0x00, 0x05, 'H', 'e', 'l', 'l', 'o']
            └─────── length ──────┘  └──── message data ────┘
```

## Customization Options

You can customize the codec behavior:

```rust
let codec = LengthDelimitedCodec::builder()
    .length_field_length(2)              // Use 2 bytes for length instead of 4
    .little_endian()                     // Use little-endian byte order
    .max_frame_length(8192)              // Set maximum frame size
    .new_codec();
```

## Benefits

1. **Automatic Framing**: No manual parsing of message boundaries
2. **Buffer Management**: Handles partial reads and writes automatically
3. **Type Safety**: Works with `Bytes` type for efficient memory handling
4. **Interoperability**: Compatible with any system using length-prefix framing
5. **Error Handling**: Detects malformed frames and oversized messages

## Common Use Cases

- Custom application protocols over TCP
- Microservice communication
- Binary message passing
- RPC implementations
- Any scenario requiring reliable message boundaries over streams

## Error Handling

The codec can return errors for:

- **I/O errors**: Network failures, connection drops
- **Frame too large**: Message exceeds `max_frame_length`
- **Incomplete frames**: Stream ended mid-message

Always use `?` operator or proper error handling to catch these cases.