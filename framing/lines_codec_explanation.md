# Understanding LinesCodec in Tokio

### Overview

`LinesCodec` is a decoder/encoder that handles newline-delimited text protocols. It automatically splits incoming TCP data into separate messages based on newline characters (`\n` or `\r\n`).


## Complete Code

```rust
use tokio::net::TcpListener;
use tokio_util::codec::{Framed, LinesCodec};
use futures::StreamExt;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let listener = TcpListener::bind("127.0.0.1:8080").await?;
    
    loop {
        let (socket, _) = listener.accept().await?;
        let mut framed = Framed::new(socket, LinesCodec::new());
        
        tokio::spawn(async move {
            while let Some(result) = framed.next().await {
                match result {
                    Ok(line) => println!("Received line: {}", line),
                    Err(e) => eprintln!("Error: {}", e),
                }
            }
        });
    }
}
```
## How LinesCodec Reads Newline-Delimited Messages

### Step-by-Step Process

#### 1. **TCP Connection Established**
```rust
let (socket, _) = listener.accept().await?;
```
When a client connects, we get a raw `TcpStream` (socket) that provides a stream of bytes.

#### 2. **Wrapping with Framed and LinesCodec**
```rust
let mut framed = Framed::new(socket, LinesCodec::new());
```
- `Framed` is a wrapper that combines a transport (the TCP socket) with a codec (`LinesCodec`)
- `LinesCodec::new()` creates a codec that knows how to parse newline-delimited text
- The result is a `Stream` of `String` values instead of raw bytes

#### 3. **Internal Buffering**

When data arrives over the TCP connection:

- **Raw bytes arrive**: The TCP stream delivers bytes as they come over the network (could be partial messages)
- **LinesCodec buffers**: It maintains an internal buffer to accumulate bytes
- **Searches for newlines**: It scans the buffer for newline characters (`\n` or `\r\n`)
- **Extracts complete lines**: When a newline is found, everything before it becomes one message

**Example:**

If the TCP stream receives bytes like this:
```
"Hello\nWor" → "ld\nHow are" → " you?\n"
```

LinesCodec processes it as:
1. First message: `"Hello"`
2. Second message: `"World"`
3. Third message: `"How are you?"`

#### 4. **Consuming Messages as a Stream**
```rust
while let Some(result) = framed.next().await {
    match result {
        Ok(line) => println!("Received line: {}", line),
        Err(e) => eprintln!("Error: {}", e),
    }
}
```

- `framed.next().await` waits for the next complete line
- Each call returns `Option<Result<String, Error>>`
  - `Some(Ok(line))` - A complete line was received
  - `Some(Err(e))` - An error occurred (invalid UTF-8, connection error, etc.)
  - `None` - The stream ended (client disconnected)

#### 5. **Asynchronous Handling**
```rust
tokio::spawn(async move {
    // Handle connection in separate task
});
```
Each connection runs in its own async task, allowing the server to handle multiple clients concurrently.

### Key Benefits

1. **Automatic Message Framing**: No need to manually search for newlines in the byte stream
2. **Buffering Handled**: Deals with partial messages arriving across multiple TCP packets
3. **UTF-8 Validation**: Ensures received data is valid UTF-8 text
4. **Stream Interface**: Clean async/await API for consuming messages

### Protocol Example

A client sending messages would look like:
```
Client sends: "HELLO\n"     → Server receives: "HELLO"
Client sends: "PING\n"      → Server receives: "PING"
Client sends: "BYE\n"       → Server receives: "BYE"
```

Each `\n` marks the end of a message, and LinesCodec automatically splits them into individual strings.

### What Happens Without LinesCodec

Without a codec, you'd need to:
- Manually read bytes from the socket
- Maintain your own buffer
- Search for newline characters
- Handle partial reads across TCP packet boundaries
- Validate UTF-8 encoding
- Extract complete messages

LinesCodec does all of this automatically!