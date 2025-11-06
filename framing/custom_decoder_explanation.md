# Custom Decoder Implementation for a Simple Protocol

This document explains how to implement a custom decoder using Tokio's `Decoder` trait for a simple binary protocol.

## Complete Code

```rust
use tokio_util::codec::Decoder;
use bytes::BytesMut;
use std::io;

// A simple protocol: first byte is message type, rest is payload
struct SimpleDecoder;

impl Decoder for SimpleDecoder {
    type Item = (u8, Vec<u8>);
    type Error = io::Error;
    
    fn decode(&mut self, src: &mut BytesMut) -> Result<Option<Self::Item>, Self::Error> {
        if src.len() < 2 {
            return Ok(None);
        }
        
        let msg_type = src[0];
        let length = src[1] as usize;
        
        if src.len() < 2 + length {
            return Ok(None);
        }
        
        src.split_to(2);
        let payload = src.split_to(length).to_vec();
        
        Ok(Some((msg_type, payload)))
    }
}

#[tokio::main]
async fn main() {
    println!("Custom decoder implemented");
}
```

## Protocol Design

This decoder implements a simple binary protocol with the following structure:

```
+----------+----------+------------------+
| Byte 0   | Byte 1   | Bytes 2..N       |
+----------+----------+------------------+
| Type     | Length   | Payload          |
+----------+----------+------------------+
```

- **Byte 0**: Message type identifier (0-255)
- **Byte 1**: Payload length (0-255 bytes)
- **Bytes 2+**: The actual payload data

## How the Decoder Works

### 1. The Decoder Trait

The `Decoder` trait from `tokio_util::codec` is designed for streaming protocols. It progressively reads bytes from a buffer and attempts to extract complete messages.

```rust
impl Decoder for SimpleDecoder {
    type Item = (u8, Vec<u8>);  // Returns tuple of (message_type, payload)
    type Error = io::Error;      // Error type for decoding failures
    
    fn decode(&mut self, src: &mut BytesMut) -> Result<Option<Self::Item>, Self::Error>
}
```

### 2. Checking for Minimum Header Size

```rust
if src.len() < 2 {
    return Ok(None);
}
```

The decoder first checks if there are at least 2 bytes available (for the type and length fields). If not enough data is present, it returns `Ok(None)`, signaling that more data needs to be read before a message can be decoded.

### 3. Reading the Header

```rust
let msg_type = src[0];
let length = src[1] as usize;
```

The decoder extracts:
- The message type from the first byte
- The payload length from the second byte

### 4. Checking for Complete Message

```rust
if src.len() < 2 + length {
    return Ok(None);
}
```

Before attempting to extract the payload, the decoder verifies that the buffer contains the complete message (header + payload). If the full message hasn't arrived yet, it returns `Ok(None)` to wait for more data.

### 5. Extracting the Message

```rust
src.split_to(2);
let payload = src.split_to(length).to_vec();
```

Once a complete message is available:
- `src.split_to(2)` removes and discards the 2-byte header from the buffer
- `src.split_to(length)` extracts the payload bytes and converts them to a `Vec<u8>`

This modifies the source buffer, removing the processed message so the next call to `decode` starts fresh.

### 6. Returning the Decoded Message

```rust
Ok(Some((msg_type, payload)))
```

The decoder returns the successfully decoded message as a tuple containing the message type and payload.

## Key Concepts

### BytesMut

`BytesMut` is a mutable byte buffer from the `bytes` crate that provides efficient buffer management for streaming data. The `split_to` method allows extracting bytes while efficiently managing the underlying buffer.

### Backpressure Handling

Returning `Ok(None)` implements natural backpressure: the decoder signals that it needs more data before it can produce a complete message. This prevents partial message processing and ensures message boundaries are respected.

### Incremental Decoding

The decoder is called repeatedly as data arrives. Each call processes as much data as possible and returns:
- `Ok(Some(item))` when a complete message is decoded
- `Ok(None)` when more data is needed
- `Err(e)` when a decoding error occurs

## Usage Example

This decoder would typically be used with Tokio's framed streams:

```rust
use tokio_util::codec::FramedRead;
use tokio::net::TcpStream;

// In an async context:
let stream = TcpStream::connect("127.0.0.1:8080").await?;
let mut framed = FramedRead::new(stream, SimpleDecoder);

while let Some(result) = framed.next().await {
    match result {
        Ok((msg_type, payload)) => {
            println!("Received message type {}: {:?}", msg_type, payload);
        }
        Err(e) => eprintln!("Decode error: {}", e),
    }
}
```

## Limitations

This simple protocol has several limitations:

1. **Payload size**: Limited to 255 bytes (one byte for length)
2. **No error checking**: No checksums or validation
3. **No fragmentation**: Messages must fit within the size limit
4. **Fixed format**: No version negotiation or extensibility

For production use, consider more robust protocols like Protocol Buffers, MessagePack, or custom protocols with proper framing and error detection.