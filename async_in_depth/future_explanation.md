# Understanding How Rust Functions Return Future Types

## Complete Code

```rust
use std::future::Future;
use std::pin::Pin;

async fn simple_future() -> i32 {
    42
}

fn returns_future() -> impl Future<Output = i32> {
    // Return the future without awaiting it
    simple_future()
}

#[tokio::main]
async fn main() {
    let result = returns_future().await;
    println!("Result: {}", result);
}
```

## How Functions Return Types Implementing `Future`

### 1. **The `async fn` Function**

```rust
async fn simple_future() -> i32 {
    42
}
```

When you declare a function with `async fn`, Rust automatically transforms it into a function that returns a type implementing `Future`. Even though the signature says it returns `i32`, it actually returns `impl Future<Output = i32>`.

**What actually happens:**
- The function doesn't execute immediately when called
- Instead, it returns a future that, when awaited, will produce an `i32`
- The Rust compiler generates a state machine under the hood

### 2. **Explicit `impl Future` Return Type**

```rust
fn returns_future() -> impl Future<Output = i32> {
    simple_future()
}
```

This function explicitly declares that it returns something implementing the `Future` trait. The `impl Future<Output = i32>` means:
- `impl` = "some type that implements"
- `Future` = the Future trait
- `Output = i32` = when this future completes, it produces an `i32`

**Key points:**
- `returns_future()` is NOT an async function (no `async` keyword)
- It returns the future created by `simple_future()` without awaiting it
- This is useful when you want to pass futures around without executing them

### 3. **Consuming the Future**

```rust
let result = returns_future().await;
```

The `.await` keyword:
- Takes the `Future` and polls it until completion
- Extracts the `Output` value (the `i32` in this case)
- Can only be used inside `async` functions or blocks

## Key Concepts

### Futures are Lazy

Futures in Rust don't do anything until they're awaited or polled. When you call `simple_future()` or `returns_future()`, you get back a future object, but no computation has happened yet.

### The Future Trait

```rust
pub trait Future {
    type Output;
    
    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output>;
}
```

Any type implementing this trait can be awaited. The `Output` associated type determines what value the future produces when it completes.

### Why Use `impl Future`?

There are several reasons to return `impl Future` explicitly:

1. **Flexibility**: You can return different future types from the same function
2. **Composition**: You can combine or transform futures before returning them
3. **Clarity**: Makes it explicit that you're returning a lazy computation
4. **Control**: Allows the caller to decide when to execute the future

## Execution Flow

1. `returns_future()` is called → Returns a future (not executed yet)
2. `.await` is called on that future → Starts execution
3. `simple_future()` executes → Produces the value `42`
4. The value `42` is assigned to `result`
5. The value is printed

## Alternative: Using Box<dyn Future>

If you need dynamic dispatch or want to store futures of different types, you can use:

```rust
fn returns_boxed_future() -> Pin<Box<dyn Future<Output = i32>>> {
    Box::pin(simple_future())
}
```

This uses trait objects instead of `impl Trait`, which is more flexible but has runtime overhead.