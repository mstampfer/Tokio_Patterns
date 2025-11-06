# Returning Different Future Types Using Trait Objects in Rust

## Complete Code

```rust
use std::future::Future;
use std::pin::Pin;

async fn fast_operation() -> String {
    "Fast".to_string()
}

async fn slow_operation() -> String {
    tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
    "Slow".to_string()
}

fn get_operation(fast: bool) -> Pin<Box<dyn Future<Output = String>>> {
    if fast {
        Box::pin(fast_operation())
    } else {
        Box::pin(slow_operation())
    }
}

#[tokio::main]
async fn main() {
    let result = get_operation(true).await;
    println!("Result: {}", result);
    
    let result2 = get_operation(false).await;
    println!("Result: {}", result2);
}
```

## How This Code Returns Different Future Types

### The Problem: Different Concrete Types

Each `async fn` creates a **unique, anonymous future type**. Even though both functions have the same signature, they generate different types:

```rust
async fn fast_operation() -> String { ... }  // Creates FutureType1
async fn slow_operation() -> String { ... }  // Creates FutureType2
```

These are **different concrete types** at compile time, even though they both implement `Future<Output = String>`.

### Why `impl Future` Won't Work Here

If we tried to use `impl Future`:

```rust
// This won't compile!
fn get_operation(fast: bool) -> impl Future<Output = String> {
    if fast {
        fast_operation()  // Returns FutureType1
    } else {
        slow_operation()  // Returns FutureType2 - ERROR!
    }
}
```

**Error:** The compiler requires that all branches return the exact same concrete type. `impl Future` means "some specific type that implements Future," but it must be the same type in all code paths.

### The Solution: Trait Objects with `dyn Future`

```rust
fn get_operation(fast: bool) -> Pin<Box<dyn Future<Output = String>>> {
    if fast {
        Box::pin(fast_operation())
    } else {
        Box::pin(slow_operation())
    }
}
```

This works because we're using **dynamic dispatch** through trait objects:

- `dyn Future<Output = String>` = "any type that implements Future<Output = String>"
- `Box<dyn Future<...>>` = heap-allocated trait object
- `Pin<Box<...>>` = ensures the future doesn't move in memory

## Breaking Down the Return Type

### `Pin<Box<dyn Future<Output = String>>>`

Let's examine each component:

#### 1. `dyn Future<Output = String>`

- `dyn` = dynamic dispatch (runtime polymorphism)
- Allows different concrete types to be treated as the same trait object
- The actual type is determined at runtime, not compile time

#### 2. `Box<...>`

- Heap allocation is required for trait objects
- Trait objects are unsized types (unknown size at compile time)
- `Box` provides a fixed-size pointer to the heap-allocated data

#### 3. `Pin<...>`

- Futures in Rust must not move in memory once they start being polled
- `Pin` ensures the future stays at the same memory address
- Required for safe async/await operations

## How It Works at Runtime

### Step 1: Function Call

```rust
let result = get_operation(true).await;
```

1. `get_operation(true)` is called
2. The function evaluates the condition (`fast == true`)
3. Takes the `if` branch
4. Calls `fast_operation()` which returns a future
5. Wraps it in `Box::pin(...)` creating a trait object
6. Returns `Pin<Box<dyn Future<Output = String>>>`

### Step 2: Awaiting

```rust
.await
```

1. The `.await` polls the future through the trait object
2. Uses **dynamic dispatch** - calls the `poll` method via a vtable
3. The actual future type (`fast_operation`'s future) executes
4. Returns `"Fast".to_string()` when complete

### Step 3: Different Path

```rust
let result2 = get_operation(false).await;
```

1. `get_operation(false)` is called
2. Takes the `else` branch this time
3. Calls `slow_operation()` which returns a **different** future type
4. Also wrapped in `Box::pin(...)`
5. Returns the same return type: `Pin<Box<dyn Future<Output = String>>>`
6. When awaited, executes the slow operation with the sleep

## Comparison: Static vs Dynamic Dispatch

### Static Dispatch (`impl Future`)

```rust
// Each function must return ONE specific type
fn only_fast() -> impl Future<Output = String> {
    fast_operation()  // Always returns the same concrete type
}
```

**Advantages:**
- ✅ Zero-cost abstraction (no runtime overhead)
- ✅ Compiler can inline and optimize
- ✅ No heap allocation needed

**Disadvantages:**
- ❌ Cannot return different types from different branches
- ❌ Cannot store in collections with mixed future types

### Dynamic Dispatch (`dyn Future`)

```rust
fn get_operation(fast: bool) -> Pin<Box<dyn Future<Output = String>>> {
    if fast {
        Box::pin(fast_operation())   // Different type
    } else {
        Box::pin(slow_operation())   // Different type
    }
}
```

**Advantages:**
- ✅ Can return different concrete types
- ✅ Can store mixed types in collections
- ✅ More flexible API design

**Disadvantages:**
- ❌ Heap allocation required (`Box`)
- ❌ Runtime overhead (vtable lookup)
- ❌ Cannot be inlined by compiler

## Real-World Use Cases

### 1. Conditional Logic

```rust
fn fetch_data(use_cache: bool) -> Pin<Box<dyn Future<Output = Data>>> {
    if use_cache {
        Box::pin(fetch_from_cache())
    } else {
        Box::pin(fetch_from_network())
    }
}
```

### 2. Strategy Pattern

```rust
fn get_handler(strategy: Strategy) -> Pin<Box<dyn Future<Output = Response>>> {
    match strategy {
        Strategy::Fast => Box::pin(fast_handler()),
        Strategy::Reliable => Box::pin(reliable_handler()),
        Strategy::Balanced => Box::pin(balanced_handler()),
    }
}
```

### 3. Plugin Systems

```rust
fn load_plugin(name: &str) -> Pin<Box<dyn Future<Output = Plugin>>> {
    match name {
        "auth" => Box::pin(load_auth_plugin()),
        "logging" => Box::pin(load_logging_plugin()),
        _ => Box::pin(load_default_plugin()),
    }
}
```

## Performance Considerations

### Memory Layout

```
Stack:           Heap:
┌─────────┐     ┌──────────────────┐
│  Pin    │     │  Future Type A   │
│  ├─Box──┼────>│  (vtable ptr)    │
│         │     │  (actual data)   │
└─────────┘     └──────────────────┘
```

- The `Pin<Box<...>>` itself lives on the stack (fixed size pointer)
- The actual future lives on the heap
- An extra vtable pointer enables dynamic dispatch

### Cost Analysis

1. **Allocation**: One heap allocation per future creation
2. **Indirection**: One extra pointer dereference when polling
3. **Vtable lookup**: Method calls go through virtual dispatch

For most applications, this overhead is acceptable. Only optimize if profiling shows this is a bottleneck.

## Alternative: Enum-Based Approach

If you know all possible future types at compile time, you can use an enum:

```rust
enum Operation {
    Fast(impl Future<Output = String>),
    Slow(impl Future<Output = String>),
}

// But this gets complicated fast and isn't always practical
```

Trait objects are usually the cleaner solution for returning different future types.

## Summary

This code demonstrates how to:
- Return different future types from a single function
- Use `dyn Future` for dynamic dispatch
- Properly box and pin futures for safe async operations
- Trade compile-time optimization for runtime flexibility

The key insight is that while `impl Future` requires a single concrete type, `dyn Future` allows any type implementing the Future trait, enabling true polymorphism in async Rust code.