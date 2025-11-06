# Handling Partial Frames in a Custom Decoder

## Overview

This decoder implements a length-prefixed protocol that gracefully handles **partial frames** - situations where a complete message hasn't arrived yet over the network. This is a fundamental challenge in TCP programming because data arrives in chunks, not necessarily aligned with message boundaries.


## Complete Code

```rust
use tokio_util::codec::Decoder;
use bytes::BytesMut;
use std::io;

struct MessageDecoder;

impl Decoder for MessageDecoder {
    type Item = String;
    type Error = io::Error;
    
    fn decode(&mut self, src: &mut BytesMut) -> Result<Option<Self::Item>, Self::Error> {
        // Protocol: 4-byte length prefix (big-endian) followed by UTF-8 string
        
        // Check 1: Do we have at least 4 bytes for the length prefix?
        if src.len() < 4 {
            // Not enough data yet - return None and wait for more
            return Ok(None);
        }
        
        // Check 2: Parse the length and verify we have the complete message
        let length = u32::from_be_bytes([src[0], src[1], src[2], src[3]]) as usize;
        
        // Not enough data yet for the complete message
        if src.len() < 4 + length {
            // Reserve more space in the buffer (optimization)
            src.reserve(4 + length - src.len());
            return Ok(None);
        }
        
        // Check 3: We have a complete frame - decode it!
        
        // Remove the length prefix from the buffer
        src.split_to(4);
        
        // Extract exactly 'length' bytes for the message
        let message_bytes = src.split_to(length);
        
        // Convert bytes to UTF-8 string, propagating errors
        let message = String::from_utf8(message_bytes.to_vec())
            .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;
        
        // Return the decoded message
        Ok(Some(message))
    }
}

#[tokio::main]
async fn main() {
    println!("Decoder handles partial frames");
}
```


## The Protocol

The decoder expects messages in this format:

```
┌─────────────┬──────────────────────────┐
│  4 bytes    │    N bytes               │
│  (length)   │    (UTF-8 string)        │
├─────────────┼──────────────────────────┤
│ 0x00000005  │  "hello"                 │
└─────────────┴──────────────────────────┘
   Big-endian      Actual message data
   uint32
```

## The Partial Frame Problem

### Scenario: Data Arrives in Chunks

When you receive data over TCP, it might arrive like this:

**Message to send:** `[0, 0, 0, 5, 'h', 'e', 'l', 'l', 'o']` (9 bytes total)

**But it might arrive as:**
- **Chunk 1:** `[0, 0]` (only 2 bytes - incomplete length prefix!)
- **Chunk 2:** `[0, 5, 'h', 'e']` (4 more bytes - length is complete, but message is not!)
- **Chunk 3:** `['l', 'l', 'o']` (final 3 bytes - now we have everything!)

The decoder must handle all three scenarios:
1. **Not enough data for the length prefix** (need at least 4 bytes)
2. **Have the length, but not all the message data** (need 4 + length bytes)
3. **Have a complete frame** (can decode and return the message)

## How the Code Handles Partial Frames

### State Machine Approach

The decoder acts as a state machine with three states:

```
┌─────────────────────────────────────────────────────────┐
│                                                         │
│  ┌──────────────────┐                                  │
│  │ State 1:         │                                  │
│  │ Waiting for      │───► src.len() < 4                │
│  │ length prefix    │    Return Ok(None)               │
│  └──────────────────┘    "Need more data"              │
│           │                                             │
│           │ src.len() >= 4                              │
│           ▼                                             │
│  ┌──────────────────┐                                  │
│  │ State 2:         │                                  │
│  │ Waiting for      │───► src.len() < 4 + length       │
│  │ complete message │    Return Ok(None)               │
│  └──────────────────┘    "Need more data"              │
│           │                                             │
│           │ src.len() >= 4 + length                     │
│           ▼                                             │
│  ┌──────────────────┐                                  │
│  │ State 3:         │                                  │
│  │ Complete frame   │───► Decode and return            │
│  │ available        │    Ok(Some(message))             │
│  └──────────────────┘                                  │
│                                                         │
└─────────────────────────────────────────────────────────┘
```

### Code Walkthrough

#### Check 1: Do we have enough bytes for the length prefix?

```rust
if src.len() < 4 {
    return Ok(None);
}
```

**What happens:**
- The decoder checks if there are at least 4 bytes in the buffer
- If not, it returns `Ok(None)` which tells Tokio: *"I can't decode anything yet, call me again when more data arrives"*
- The data stays in the buffer untouched

**Example:**
```rust
// Buffer contains: [0, 0]
// src.len() = 2, which is < 4
// Returns: Ok(None)
// Buffer remains: [0, 0] (waiting for more data)
```

#### Check 2: Parse the length and check if we have the complete message

```rust
let length = u32::from_be_bytes([src[0], src[1], src[2], src[3]]) as usize;

if src.len() < 4 + length {
    // Reserve more space
    src.reserve(4 + length - src.len());
    return Ok(None);
}
```

**What happens:**
- The decoder reads the 4-byte length prefix (big-endian format)
- It checks if the buffer has `4 + length` bytes (length prefix + message data)
- If not enough data, it reserves space in the buffer for efficiency
- Returns `Ok(None)` again - *"I know how much data I need, but don't have it all yet"*

**Example:**
```rust
// Buffer contains: [0, 0, 0, 5, 'h', 'e']
// Length parsed: 5
// Need: 4 + 5 = 9 bytes
// Have: 6 bytes
// src.len() (6) < 4 + length (9)
// Returns: Ok(None)
// Buffer remains: [0, 0, 0, 5, 'h', 'e'] (waiting for 3 more bytes)
```

**The `reserve()` call:**
```rust
src.reserve(4 + length - src.len());
```
This is an optimization that pre-allocates buffer space. If we know we need 9 bytes total and have 6, we reserve 3 more bytes to avoid multiple reallocations.

#### Check 3: We have a complete frame - decode it!

```rust
// Remove the length prefix
src.split_to(4);

// Extract the message
let message_bytes = src.split_to(length);
let message = String::from_utf8(message_bytes.to_vec())
    .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;

Ok(Some(message))
```

**What happens:**
- `src.split_to(4)` removes and discards the 4-byte length prefix
- `src.split_to(length)` extracts exactly `length` bytes for the message
- The message bytes are converted from UTF-8 to a `String`
- Returns `Ok(Some(message))` - *"Here's your decoded message!"*

**Example:**
```rust
// Buffer contains: [0, 0, 0, 5, 'h', 'e', 'l', 'l', 'o']
// Length parsed: 5
// Have: 9 bytes, need: 9 bytes ✓
// 
// After src.split_to(4):
//   Buffer: ['h', 'e', 'l', 'l', 'o']
// 
// After src.split_to(5):
//   message_bytes: ['h', 'e', 'l', 'l', 'o']
//   Buffer: [] (empty, ready for next message)
// 
// Returns: Ok(Some("hello"))
```

## Visual Example: Processing a Stream

Let's trace through a realistic scenario where data arrives in multiple chunks:

### Initial State
```
Buffer: []
```

### Chunk 1 arrives: `[0, 0]`
```rust
decode() called
├─ src.len() = 2
├─ Check: 2 < 4? YES
└─ Return: Ok(None)

Buffer after: [0, 0]
Status: Waiting for more data
```

### Chunk 2 arrives: `[0, 5, 'h', 'e']`
```rust
Buffer before: [0, 0, 0, 5, 'h', 'e']

decode() called
├─ src.len() = 6
├─ Check: 6 < 4? NO, continue
├─ Parse length: u32::from_be_bytes([0, 0, 0, 5]) = 5
├─ Check: 6 < 4 + 5 (9)? YES
├─ Reserve: 9 - 6 = 3 more bytes
└─ Return: Ok(None)

Buffer after: [0, 0, 0, 5, 'h', 'e']
Status: Waiting for 3 more bytes
```

### Chunk 3 arrives: `['l', 'l', 'o']`
```rust
Buffer before: [0, 0, 0, 5, 'h', 'e', 'l', 'l', 'o']

decode() called
├─ src.len() = 9
├─ Check: 9 < 4? NO, continue
├─ Parse length: 5
├─ Check: 9 < 4 + 5 (9)? NO, we have enough!
├─ split_to(4): Remove length prefix
│   Buffer now: ['h', 'e', 'l', 'l', 'o']
├─ split_to(5): Extract message
│   message_bytes: ['h', 'e', 'l', 'l', 'o']
│   Buffer now: []
├─ Convert to String: "hello"
└─ Return: Ok(Some("hello"))

Buffer after: []
Status: Message decoded successfully!
```

## Multiple Messages in the Buffer

The decoder can handle multiple messages in the buffer. After decoding one message, the buffer might still contain data for the next message:

```rust
// Buffer: [0, 0, 0, 5, 'h', 'e', 'l', 'l', 'o', 0, 0, 0, 5, 'w', 'o', 'r', 'l', 'd']
//         └──────── Message 1 ────────────┘ └──────── Message 2 ────────────┘

// First decode() call:
//   - Extracts and returns "hello"
//   - Buffer after: [0, 0, 0, 5, 'w', 'o', 'r', 'l', 'd']

// Second decode() call:
//   - Extracts and returns "world"
//   - Buffer after: []
```

Tokio will keep calling `decode()` until it returns `Ok(None)`, ensuring all complete messages are extracted.

## Key Design Principles

### 1. **Non-Destructive Reading**

When returning `Ok(None)`, the buffer is **not modified**. This is crucial because:
- We don't know when more data will arrive
- We can't partially consume a message
- The next call to `decode()` will see the same data plus new data

### 2. **Eager Decoding**

The decoder extracts as many complete messages as possible in a single call. If the buffer contains:
```
[msg1][msg2][msg3-partial]
```

The decoder will:
- Return `msg1` on first call
- Return `msg2` on second call (Tokio calls again automatically)
- Return `Ok(None)` on third call (partial message remains in buffer)

### 3. **Buffer Management**

```rust
src.reserve(4 + length - src.len());
```

This optimization:
- Prevents multiple allocations as data trickles in
- Allocates exactly the space needed for the complete message
- Improves performance for large messages

### 4. **Zero-Copy When Possible**

```rust
src.split_to(4);  // Discards length prefix
let message_bytes = src.split_to(length);  // Extracts message
```

`split_to()` is efficient - it doesn't copy data unnecessarily. It manipulates buffer pointers internally.

## Error Handling

The decoder handles UTF-8 decoding errors gracefully:

```rust
let message = String::from_utf8(message_bytes.to_vec())
    .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?;
```

If the message bytes aren't valid UTF-8:
- The error is converted to `io::Error` with kind `InvalidData`
- The `?` operator propagates the error up
- Tokio will close the connection or handle the error according to your error handling logic


## Testing the Decoder

Here's a simple test to demonstrate partial frame handling:

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use tokio_util::codec::Decoder;
    use bytes::BytesMut;

    #[test]
    fn test_partial_frames() {
        let mut decoder = MessageDecoder;
        let mut buf = BytesMut::new();
        
        // Simulate: only 2 bytes of length prefix arrived
        buf.extend_from_slice(&[0, 0]);
        assert_eq!(decoder.decode(&mut buf).unwrap(), None);
        assert_eq!(buf.len(), 2); // Buffer unchanged
        
        // More data arrives: complete length + partial message
        buf.extend_from_slice(&[0, 5, b'h', b'e']);
        assert_eq!(decoder.decode(&mut buf).unwrap(), None);
        assert_eq!(buf.len(), 6); // Buffer still has partial data
        
        // Final data arrives: rest of message
        buf.extend_from_slice(&[b'l', b'l', b'o']);
        assert_eq!(decoder.decode(&mut buf).unwrap(), Some("hello".to_string()));
        assert_eq!(buf.len(), 0); // Buffer now empty
    }
    
    #[test]
    fn test_multiple_messages() {
        let mut decoder = MessageDecoder;
        let mut buf = BytesMut::new();
        
        // Two complete messages in buffer
        buf.extend_from_slice(&[0, 0, 0, 2, b'h', b'i']);
        buf.extend_from_slice(&[0, 0, 0, 3, b'b', b'y', b'e']);
        
        // Decode first message
        assert_eq!(decoder.decode(&mut buf).unwrap(), Some("hi".to_string()));
        
        // Decode second message
        assert_eq!(decoder.decode(&mut buf).unwrap(), Some("bye".to_string()));
        
        // No more messages
        assert_eq!(decoder.decode(&mut buf).unwrap(), None);
    }
}
```

## Summary

The decoder handles partial frames through a **three-stage checking process**:

1. **Stage 1:** Check if we have the length prefix (4 bytes)
   - If not → `Ok(None)` - wait for more data
   
2. **Stage 2:** Check if we have the complete message (4 + length bytes)
   - If not → `Ok(None)` - wait for more data
   - Optimize by pre-allocating buffer space
   
3. **Stage 3:** Decode the complete message
   - Extract and remove the consumed bytes
   - Convert to string and return `Ok(Some(message))`

This pattern ensures that:
- No data is lost or corrupted
- The decoder is efficient (minimal copying/allocation)
- Multiple messages can be decoded from a single buffer
- The system is robust against network timing variations

The key insight is that **`Ok(None)` is not an error** - it's a normal part of the protocol that means "I need more data before I can decode."