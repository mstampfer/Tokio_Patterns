# Complete Codec Implementation: Encoder and Decoder

## Overview

This code demonstrates how to create a unified codec struct that implements both the `Encoder` and `Decoder` traits from Tokio. This allows for bidirectional communication over network connections, enabling both sending and receiving messages using the same protocol format.

## What is a Codec?

A **codec** (coder-decoder) is a component that:
- **Encodes**: Converts structured data into a byte stream for transmission
- **Decodes**: Parses a byte stream back into structured data

By implementing both traits on a single struct, you create a complete protocol handler that can be used with Tokio's `Framed` wrapper for seamless async I/O.

## Protocol Format

The protocol uses the same simple format for both encoding and decoding:

```
+-------------+----------------+------------------+
| Message Type | Payload Length |     Payload      |
|   (1 byte)   |    (1 byte)    |  (0-255 bytes)   |
+-------------+----------------+------------------+
```

## Complete Code

```rust
use tokio_util::codec::{Decoder, Encoder};
use bytes::BytesMut;
use std::io;

struct SimpleCodec;

impl Decoder for SimpleCodec {
    type Item = (u8, Vec<u8>);
    type Error = io::Error;
    
    fn decode(&mut self, src: &mut BytesMut) -> Result<Option<Self::Item>, Self::Error> {
        // Check if we have enough bytes for the header
        if src.len() < 2 {
            return Ok(None);
        }
        
        // Read the header
        let msg_type = src[0];
        let length = src[1] as usize;
        
        // Check if we have the complete message
        if src.len() < 2 + length {
            return Ok(None);
        }
        
        // Remove the header bytes
        let _ = src.split_to(2);
        
        // Extract the payload
        let payload = src.split_to(length).to_vec();
        
        Ok(Some((msg_type, payload)))
    }
}

impl Encoder<(u8, Vec<u8>)> for SimpleCodec {
    type Error = io::Error;
    
    fn encode(&mut self, item: (u8, Vec<u8>), dst: &mut BytesMut) -> Result<(), Self::Error> {
        let (msg_type, payload) = item;
        
        // Validate payload size
        if payload.len() > 255 {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "Payload too large"
            ));
        }
        
        // Write header and payload
        dst.extend_from_slice(&[msg_type, payload.len() as u8]);
        dst.extend_from_slice(&payload);
        
        Ok(())
    }
}

use tokio::net::TcpStream;
use tokio_util::codec::Framed;
use futures::{SinkExt, StreamExt};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let stream = TcpStream::connect("127.0.0.1:8080").await?;
    let mut framed = Framed::new(stream, SimpleCodec);
    
    // Send a message
    let message = (1u8, b"Hello".to_vec());
    framed.send(message).await?;
    
    Ok(())
}
```

## Detailed Component Breakdown

### 1. The Codec Struct

```rust
struct SimpleCodec;
```

This is a zero-sized type (ZST) that serves as the codec implementation. It contains no data because the protocol is stateless. For stateful protocols (like compression or encryption), you would add fields here.

### 2. Decoder Implementation

```rust
impl Decoder for SimpleCodec {
    type Item = (u8, Vec<u8>);
    type Error = io::Error;
    
    fn decode(&mut self, src: &mut BytesMut) -> Result<Option<Self::Item>, Self::Error>
```

#### Associated Types
- **`type Item`**: The type of decoded messages - a tuple of message type and payload
- **`type Error`**: The error type for decoding failures

#### The decode Method

The `decode` method is called repeatedly as data arrives. It returns:
- `Ok(Some(item))`: A complete message was decoded
- `Ok(None)`: Not enough data yet, need more bytes
- `Err(e)`: A decoding error occurred

#### Decoding Steps

**Step 1: Check for Header**
```rust
if src.len() < 2 {
    return Ok(None);
}
```
If we don't have at least 2 bytes (message type + length), we return `None` to wait for more data.

**Step 2: Read Header**
```rust
let msg_type = src[0];
let length = src[1] as usize;
```
Extract the message type and payload length from the buffer without consuming the bytes yet.

**Step 3: Check for Complete Message**
```rust
if src.len() < 2 + length {
    return Ok(None);
}
```
Verify we have the complete message (header + full payload).

**Step 4: Extract the Message**
```rust
let _ = src.split_to(2);
let payload = src.split_to(length).to_vec();
```
- `split_to(2)`: Removes and discards the header bytes
- `split_to(length)`: Removes and returns the payload bytes
- `.to_vec()`: Converts `BytesMut` to `Vec<u8>`

**Step 5: Return the Decoded Message**
```rust
Ok(Some((msg_type, payload)))
```

### 3. Encoder Implementation

```rust
impl Encoder<(u8, Vec<u8>)> for SimpleCodec {
    type Error = io::Error;
    
    fn encode(&mut self, item: (u8, Vec<u8>), dst: &mut BytesMut) -> Result<(), Self::Error>
```

#### The encode Method

Takes a message tuple and writes it to the destination buffer.

**Step 1: Validate Payload**
```rust
if payload.len() > 255 {
    return Err(io::Error::new(
        io::ErrorKind::InvalidInput,
        "Payload too large"
    ));
}
```
Ensures the payload fits within the 1-byte length field.

**Step 2: Write Header and Payload**
```rust
dst.extend_from_slice(&[msg_type, payload.len() as u8]);
dst.extend_from_slice(&payload);
```
Writes the message type, length, and payload sequentially.

### 4. Using the Codec with Framed

```rust
let stream = TcpStream::connect("127.0.0.1:8080").await?;
let mut framed = Framed::new(stream, SimpleCodec);
```

`Framed` wraps a byte stream (like `TcpStream`) and a codec to provide:
- **Sink**: For sending messages (uses the `Encoder`)
- **Stream**: For receiving messages (uses the `Decoder`)

#### Sending Messages

```rust
let message = (1u8, b"Hello".to_vec());
framed.send(message).await?;
```

The `send` method:
1. Calls the `Encoder::encode` method
2. Writes the encoded bytes to the TCP stream
3. Returns when the write completes

#### Receiving Messages

```rust
while let Some(result) = framed.next().await {
    match result {
        Ok((msg_type, payload)) => {
            println!("Received message type {}: {:?}", msg_type, payload);
        }
        Err(e) => {
            eprintln!("Error decoding message: {}", e);
            break;
        }
    }
}
```

The `next` method:
1. Reads bytes from the TCP stream
2. Calls `Decoder::decode` repeatedly
3. Returns decoded messages as they become available

## Complete Client-Server Example

### Server

```rust
use tokio::net::TcpListener;
use tokio_util::codec::Framed;
use futures::{SinkExt, StreamExt};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let listener = TcpListener::bind("127.0.0.1:8080").await?;
    println!("Server listening on 127.0.0.1:8080");
    
    loop {
        let (socket, addr) = listener.accept().await?;
        println!("New connection from: {}", addr);
        
        tokio::spawn(async move {
            let mut framed = Framed::new(socket, SimpleCodec);
            
            // Receive messages
            while let Some(result) = framed.next().await {
                match result {
                    Ok((msg_type, payload)) => {
                        println!("Received type {}: {:?}", msg_type, payload);
                        
                        // Echo back
                        let response = (msg_type, payload);
                        if let Err(e) = framed.send(response).await {
                            eprintln!("Error sending response: {}", e);
                            break;
                        }
                    }
                    Err(e) => {
                        eprintln!("Error decoding: {}", e);
                        break;
                    }
                }
            }
        });
    }
}
```

### Client

```rust
use tokio::net::TcpStream;
use tokio_util::codec::Framed;
use futures::{SinkExt, StreamExt};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let stream = TcpStream::connect("127.0.0.1:8080").await?;
    let mut framed = Framed::new(stream, SimpleCodec);
    
    // Send a message
    let message = (1u8, b"Hello, Server!".to_vec());
    framed.send(message).await?;
    println!("Message sent");
    
    // Receive the response
    if let Some(result) = framed.next().await {
        match result {
            Ok((msg_type, payload)) => {
                let text = String::from_utf8_lossy(&payload);
                println!("Received type {}: {}", msg_type, text);
            }
            Err(e) => {
                eprintln!("Error: {}", e);
            }
        }
    }
    
    Ok(())
}
```

## Key Advantages of This Approach

### 1. Type Safety
The codec enforces the protocol at compile time. You can't accidentally send malformed messages.

### 2. Automatic Framing
The decoder handles partial messages automatically. You don't need to manually manage buffering.

### 3. Bidirectional Communication
One struct handles both directions, ensuring consistency between encoding and decoding.

### 4. Integration with Tokio Ecosystem
Works seamlessly with `Stream` and `Sink` traits, enabling composition with other async utilities.

### 5. Backpressure Handling
The `Framed` wrapper automatically handles backpressure, preventing memory exhaustion.

## Common Patterns and Extensions

### Pattern 1: Stateful Codec

For protocols requiring state (like compression):

```rust
struct StatefulCodec {
    compression_level: u8,
    message_count: u64,
}

impl StatefulCodec {
    fn new(compression_level: u8) -> Self {
        Self {
            compression_level,
            message_count: 0,
        }
    }
}
```

### Pattern 2: Multiple Message Types

```rust
enum Message {
    Ping,
    Pong,
    Data(Vec<u8>),
    Error(String),
}

impl Decoder for MessageCodec {
    type Item = Message;
    type Error = io::Error;
    
    fn decode(&mut self, src: &mut BytesMut) -> Result<Option<Self::Item>, Self::Error> {
        // Decode based on message type byte
        match msg_type {
            0 => Ok(Some(Message::Ping)),
            1 => Ok(Some(Message::Pong)),
            2 => Ok(Some(Message::Data(payload))),
            3 => {
                let error_msg = String::from_utf8_lossy(&payload).to_string();
                Ok(Some(Message::Error(error_msg)))
            }
            _ => Err(io::Error::new(io::ErrorKind::InvalidData, "Unknown message type")),
        }
    }
}
```

### Pattern 3: Length-Prefixed with Larger Payloads

For payloads larger than 255 bytes:

```rust
fn decode(&mut self, src: &mut BytesMut) -> Result<Option<Self::Item>, Self::Error> {
    if src.len() < 5 {
        return Ok(None);
    }
    
    let msg_type = src[0];
    let length = u32::from_be_bytes([src[1], src[2], src[3], src[4]]) as usize;
    
    if src.len() < 5 + length {
        return Ok(None);
    }
    
    let _ = src.split_to(5);
    let payload = src.split_to(length).to_vec();
    
    Ok(Some((msg_type, payload)))
}
```

## Error Handling Best Practices

### 1. Validate Early
```rust
if payload.len() > MAX_PAYLOAD_SIZE {
    return Err(io::Error::new(
        io::ErrorKind::InvalidInput,
        format!("Payload too large: {} bytes", payload.len())
    ));
}
```

### 2. Provide Context
```rust
Err(io::Error::new(
    io::ErrorKind::InvalidData,
    format!("Invalid message type: {}", msg_type)
))
```

### 3. Handle Partial Reads Gracefully
Always return `Ok(None)` when you need more data, never panic.

## Performance Considerations

### 1. Buffer Management
```rust
// Reserve space to avoid reallocations
dst.reserve(2 + payload.len());
```

### 2. Zero-Copy Operations
Use `split_to` instead of copying data when possible.

### 3. Avoid Unnecessary Allocations
```rust
// Instead of .to_vec() if you can work with BytesMut
let payload = src.split_to(length); // Returns BytesMut
```

## Testing Your Codec

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use bytes::BytesMut;
    
    #[test]
    fn test_encode_decode() {
        let mut codec = SimpleCodec;
        let mut buf = BytesMut::new();
        
        // Encode a message
        let msg = (1u8, vec![1, 2, 3, 4, 5]);
        codec.encode(msg.clone(), &mut buf).unwrap();
        
        // Decode it back
        let decoded = codec.decode(&mut buf).unwrap();
        assert_eq!(decoded, Some(msg));
    }
    
    #[test]
    fn test_partial_message() {
        let mut codec = SimpleCodec;
        let mut buf = BytesMut::from(&[1u8, 5u8, 1, 2][..]); // Incomplete
        
        // Should return None (need more data)
        assert_eq!(codec.decode(&mut buf).unwrap(), None);
        
        // Add remaining bytes
        buf.extend_from_slice(&[3, 4, 5]);
        
        // Now should decode successfully
        let decoded = codec.decode(&mut buf).unwrap();
        assert_eq!(decoded, Some((1u8, vec![1, 2, 3, 4, 5])));
    }
}
```

## Conclusion

Implementing both `Encoder` and `Decoder` on a single codec struct provides a clean, type-safe way to handle bidirectional communication in async Rust applications. The pattern integrates seamlessly with Tokio's ecosystem and provides automatic handling of framing, backpressure, and partial messages.

This approach is used in production systems for:
- Custom network protocols
- Message brokers
- RPC systems
- Game servers
- IoT communication
- Microservice communication

By following this pattern, you get robust, maintainable, and efficient protocol implementations with minimal boilerplate.