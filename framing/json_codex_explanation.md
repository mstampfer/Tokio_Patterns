# JSON Codec with Length Prefixes

## Overview

This code creates a custom codec that combines JSON serialization with length-delimited framing. It wraps messages in a length prefix to solve the fundamental problem of TCP streams: knowing where one message ends and another begins.

## Complete Code

```rust
use tokio_util::codec::{Decoder, Encoder, LengthDelimitedCodec};
use bytes::{BytesMut, Bytes};
use serde::{Serialize, Deserialize};
use std::io;
use std::marker::PhantomData;

#[derive(Serialize, Deserialize, Debug)]
struct Message {
    id: u32,
    text: String,
}

struct JsonCodec<T> {
    inner: LengthDelimitedCodec,
    _phantom: PhantomData<T>,
}

impl<T> JsonCodec<T> {
    fn new() -> Self {
        JsonCodec {
            inner: LengthDelimitedCodec::new(),
            _phantom: PhantomData,
        }
    }
}

impl<T: for<'de> Deserialize<'de>> Decoder for JsonCodec<T> {
    type Item = T;
    type Error = io::Error;
    
    fn decode(&mut self, src: &mut BytesMut) -> Result<Option<Self::Item>, Self::Error> {
        match self.inner.decode(src)? {
            Some(bytes) => {
                let value = serde_json::from_slice(&bytes)
                    .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;
                Ok(Some(value))
            }
            None => Ok(None),
        }
    }
}

impl<T: Serialize> Encoder<T> for JsonCodec<T> {
    type Error = io::Error;
    
    fn encode(&mut self, item: T, dst: &mut BytesMut) -> Result<(), Self::Error> {
        let json = serde_json::to_vec(&item)
            .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;
        self.inner.encode(Bytes::from(json), dst)
    }
}

#[tokio::main]
async fn main() {
    let _codec: JsonCodec<Message> = JsonCodec::new();
    println!("JSON codec created");
}
```

## The Problem This Solves

When sending JSON over TCP, you can't just concatenate JSON objects because there's no clear boundary between messages:
```json
{"id":1,"text":"hello"}{"id":2,"text":"world"}
```

Without length prefixes, the receiver doesn't know where the first message ends. This codec solves that by prefixing each JSON message with its byte length.

## How It Works

### Architecture

```
┌─────────────────────────────────────────┐
│           JsonCodec<T>                  │
│  ┌───────────────────────────────────┐  │
│  │   LengthDelimitedCodec            │  │
│  │  (handles length framing)         │  │
│  └───────────────────────────────────┘  │
│  ┌───────────────────────────────────┐  │
│  │   serde_json                      │  │
│  │  (handles JSON serialization)     │  │
│  └───────────────────────────────────┘  │
└─────────────────────────────────────────┘
```

### Encoding Process

When you encode a message:

1. **Serialize to JSON**: `serde_json::to_vec()` converts your struct to a JSON byte array
2. **Add length prefix**: `LengthDelimitedCodec` prepends a 4-byte length header
3. **Write to buffer**: The complete frame is written to the output buffer

**Example:**
```
Message { id: 42, text: "hello" }
    ↓ (JSON serialization)
[123, 34, 105, 100, 34, 58, 52, 50, ...] (25 bytes)
    ↓ (Length delimiter)
[0, 0, 0, 25, 123, 34, 105, 100, 34, 58, 52, 50, ...]
 └─length──┘ └────────JSON data──────────────────┘
```

### Decoding Process

When you decode a message:

1. **Read length prefix**: `LengthDelimitedCodec` reads the 4-byte length header
2. **Extract frame**: It waits for the full message (based on length) to arrive
3. **Deserialize JSON**: `serde_json::from_slice()` converts the bytes back to your struct

**Example:**
```
[0, 0, 0, 25, 123, 34, 105, 100, 34, 58, 52, 50, ...]
 └─length──┘ └────────JSON data──────────────────┘
    ↓ (Extract 25 bytes)
[123, 34, 105, 100, 34, 58, 52, 50, ...]
    ↓ (JSON deserialization)
Message { id: 42, text: "hello" }
```

## Code Walkthrough

### 1. The Message Structure

```rust
#[derive(Serialize, Deserialize, Debug)]
struct Message {
    id: u32,
    text: String,
}
```

This is a sample message type. The codec is generic and can work with any type that implements `Serialize` and `Deserialize`.

### 2. The JsonCodec Structure

```rust
struct JsonCodec<T> {
    inner: LengthDelimitedCodec,
    _phantom: PhantomData<T>,
}
```

- **`inner`**: The `LengthDelimitedCodec` handles the length-prefix framing
- **`_phantom`**: Since we don't store `T` directly, we use `PhantomData<T>` to tell the compiler that this struct is generic over `T`

### 3. The Decoder Implementation

```rust
impl<T: for<'de> Deserialize<'de>> Decoder for JsonCodec<T> {
    type Item = T;
    type Error = io::Error;
    
    fn decode(&mut self, src: &mut BytesMut) -> Result<Option<Self::Item>, Self::Error> {
        match self.inner.decode(src)? {
            Some(bytes) => {
                let value = serde_json::from_slice(&bytes)
                    .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;
                Ok(Some(value))
            }
            None => Ok(None),
        }
    }
}
```

**How it works:**
- First, delegate to `LengthDelimitedCodec` to extract a complete frame
- If a frame is available (`Some(bytes)`), deserialize the JSON
- If no complete frame is available yet (`None`), return `None` (more data needed)
- Convert any JSON errors into `io::Error`

### 4. The Encoder Implementation

```rust
impl<T: Serialize> Encoder<T> for JsonCodec<T> {
    type Error = io::Error;
    
    fn encode(&mut self, item: T, dst: &mut BytesMut) -> Result<(), Self::Error> {
        let json = serde_json::to_vec(&item)
            .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;
        self.inner.encode(Bytes::from(json), dst)
    }
}
```

**How it works:**
- Serialize the item to a JSON byte vector
- Convert the vector to `Bytes`
- Delegate to `LengthDelimitedCodec` to add the length prefix and write to the destination buffer
- Convert any JSON errors into `io::Error`



## Cargo.toml Dependencies

```toml
[dependencies]
tokio = { version = "1", features = ["full"] }
tokio-util = { version = "0.7", features = ["codec"] }
bytes = "1"
serde = { version = "1", features = ["derive"] }
serde_json = "1"
```

## Usage Example

This codec would typically be used with Tokio's framed streams:

```rust
use tokio_util::codec::Framed;
use tokio::net::TcpStream;

// Connect to a server
let stream = TcpStream::connect("127.0.0.1:8080").await?;

// Wrap the stream with our codec
let mut framed = Framed::new(stream, JsonCodec::<Message>::new());

// Send a message
framed.send(Message { id: 1, text: "hello".to_string() }).await?;

// Receive a message
if let Some(msg) = framed.next().await {
    println!("Received: {:?}", msg?);
}
```

## Key Benefits

1. **Type Safety**: The codec is generic, working with any serializable type
2. **Framing**: Length delimiters solve the message boundary problem
3. **Composability**: Separates framing logic from serialization logic
4. **Error Handling**: Properly converts errors between layers
5. **Efficiency**: Reuses buffers and avoids unnecessary copying

## Wire Format

The actual bytes sent over the network look like this:

```
┌────────────┬─────────────────────────────┐
│  4 bytes   │      N bytes                │
│  (length)  │   (JSON data)               │
├────────────┼─────────────────────────────┤
│ 0x00000019 │ {"id":1,"text":"hello"}     │
└────────────┴─────────────────────────────┘
     25            25 bytes of JSON
```

This format allows receivers to:
- Read the length field first
- Allocate the exact buffer size needed
- Know exactly when a complete message has arrived
- Handle multiple messages in a stream correctly