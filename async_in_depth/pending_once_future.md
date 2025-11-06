# Creating a Future That Returns Pending Once Before Completing

## Complete Code

```rust
use std::future::Future;
use std::pin::Pin;
use std::task::{Context, Poll};

struct DelayedFuture {
    polled: bool,
}

impl Future for DelayedFuture {
    type Output = &'static str;
    
    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        if !self.polled {
            self.polled = true;
            cx.waker().wake_by_ref();
            Poll::Pending
        } else {
            Poll::Ready("Done")
        }
    }
}

#[tokio::main]
async fn main() {
    let future = DelayedFuture { polled: false };
    let result = future.await;
    println!("Result: {}", result);
}
```

## Understanding the Delayed Future Pattern

### The Core Concept

This future demonstrates a fundamental async pattern: **returning `Pending` to defer completion**. Unlike a future that completes immediately, this one:

1. **First poll**: Returns `Poll::Pending` (not ready yet)
2. **Second poll**: Returns `Poll::Ready("Done")` (now complete)

This simulates how real async operations work, such as:
- Waiting for I/O operations
- Waiting for timers
- Waiting for data from channels
- Waiting for network responses

## Breaking Down the Implementation

### The State Structure

```rust
struct DelayedFuture {
    polled: bool,
}
```

**Purpose:**
- `polled`: Tracks whether the future has been polled before
- Acts as a state machine with two states:
  - `false`: First poll (not ready)
  - `true`: Subsequent polls (ready)

This is the simplest possible state machine for a delayed future.

### The Poll Implementation

```rust
fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
    if !self.polled {
        self.polled = true;
        cx.waker().wake_by_ref();
        Poll::Pending
    } else {
        Poll::Ready("Done")
    }
}
```

Let's examine this step by step:

#### First Poll Path

```rust
if !self.polled {
    self.polled = true;
    cx.waker().wake_by_ref();
    Poll::Pending
}
```

**What happens:**

1. **Check state**: `!self.polled` is `true` (hasn't been polled yet)
2. **Update state**: `self.polled = true` (mark as polled)
3. **Wake the future**: `cx.waker().wake_by_ref()` (schedule another poll)
4. **Return pending**: `Poll::Pending` (tell runtime we're not ready)

#### Second Poll Path

```rust
else {
    Poll::Ready("Done")
}
```

**What happens:**

1. **Check state**: `!self.polled` is `false` (already been polled)
2. **Return ready**: `Poll::Ready("Done")` (tell runtime we're done)

## The Critical Role of the Waker

### What is `cx.waker()`?

The `Context` parameter provides a `Waker` - a mechanism to tell the async runtime "poll me again later":

```rust
cx.waker().wake_by_ref();
```

**Without this line:**
- The future returns `Poll::Pending`
- The runtime thinks "okay, I'll poll you when someone wakes you"
- **Nobody ever wakes the future**
- The `.await` hangs forever! 🔒

**With this line:**
- The future returns `Poll::Pending`
- Before returning, it wakes itself
- The runtime schedules another poll
- The future completes on the second poll ✅

### Waker Methods

```rust
cx.waker().wake_by_ref();  // Wake without consuming the waker (clone internally)
cx.waker().clone().wake(); // Clone then wake (more explicit, same effect)
```

Both achieve the same result - schedule this future to be polled again.

## Execution Flow Diagram

### State Machine Visualization

```
Initial State:
┌──────────────────┐
│ DelayedFuture    │
│ polled: false    │
└──────────────────┘
         │
         │ .await triggers first poll
         ▼
┌──────────────────────────┐
│ poll() - First Call      │
│ - Check: !polled = true  │
│ - Set: polled = true     │
│ - Wake: wake_by_ref()    │
│ - Return: Poll::Pending  │
└──────────────────────────┘
         │
         │ Runtime schedules next poll
         ▼
┌──────────────────┐
│ DelayedFuture    │
│ polled: true     │
└──────────────────┘
         │
         │ Second poll triggered by waker
         ▼
┌──────────────────────────┐
│ poll() - Second Call     │
│ - Check: !polled = false │
│ - Return: Poll::Ready    │
└──────────────────────────┘
         │
         │ .await completes
         ▼
    result = "Done"
```

### Timeline View

```
Time    Event                           State
────────────────────────────────────────────────────────────
T0      future created                  polled: false
T1      future.await called             
T2      poll() #1 invoked              polled: false → true
T3      returns Poll::Pending          
T4      wake_by_ref() schedules poll   
T5      runtime picks up wake signal   
T6      poll() #2 invoked              polled: true
T7      returns Poll::Ready("Done")    
T8      result = "Done"                
T9      println! executes              
```

## Why Return Pending First?

### Purpose of the Delay

This pattern is useful for:

1. **Simulating async behavior**: Testing async code without real I/O
2. **Yielding to the runtime**: Allowing other tasks to run
3. **Multi-step operations**: Representing work that takes multiple polls
4. **Educational purposes**: Understanding how futures work internally

### Comparison with Real Async Operations

**Our DelayedFuture (simulated):**
```rust
First poll  → Poll::Pending  (artificially delay)
Second poll → Poll::Ready    (complete)
```

**Real timer (actual async):**
```rust
First poll  → Poll::Pending  (timer not expired)
...
Many polls  → Poll::Pending  (still waiting...)
...
Final poll  → Poll::Ready    (timer expired!)
```

**Real I/O operation (actual async):**
```rust
First poll  → Poll::Pending  (register interest, no data yet)
...
Later poll  → Poll::Ready    (data arrived!)
```

## Complete Execution Trace

Let's trace through the entire execution:

### Step 1: Initialization

```rust
let future = DelayedFuture { polled: false };
```

**Memory state:**
```
DelayedFuture {
    polled: false
}
```

### Step 2: Start Awaiting

```rust
let result = future.await;
```

The runtime begins polling the future.

### Step 3: First Poll

**Call:** `future.poll(cx)`

**Execution:**
```rust
// Enter poll method
// polled is false, so enter if block

if !self.polled {           // true (polled is false)
    self.polled = true;     // Set to true
    cx.waker().wake_by_ref(); // Schedule another poll
    Poll::Pending           // Return pending
}
```

**Result:** Returns `Poll::Pending`

**Memory state after first poll:**
```
DelayedFuture {
    polled: true  // Changed!
}
```

**Runtime action:** 
- Sees `Poll::Pending`
- Checks if a waker was called (yes, we called `wake_by_ref()`)
- Schedules the future to be polled again

### Step 4: Second Poll

**Call:** `future.poll(cx)` (triggered by the waker)

**Execution:**
```rust
// Enter poll method
// polled is true, so skip if block

if !self.polled {           // false (polled is true)
    // Skip this block
} else {
    Poll::Ready("Done")     // Return ready with value
}
```

**Result:** Returns `Poll::Ready("Done")`

**Runtime action:**
- Sees `Poll::Ready("Done")`
- Extracts the value `"Done"`
- Completes the `.await` expression
- Assigns `"Done"` to `result`

### Step 5: Completion

```rust
println!("Result: {}", result);
// Prints: Result: Done
```

## The Waker in Detail

### Understanding `wake_by_ref()`

```rust
cx.waker().wake_by_ref();
```

**What it does:**
1. Accesses the waker from the context
2. Signals the runtime: "This future needs to be polled again"
3. Doesn't consume the waker (borrows it temporarily)
4. Returns control to the caller

**Why it's needed:**
- Futures are **lazy** - they only make progress when polled
- After returning `Pending`, the runtime won't poll again unless told to
- The waker is how futures tell the runtime "I'm ready for another poll"

### When to Wake

**Typical patterns:**

```rust
// Immediate wake (like our example)
cx.waker().wake_by_ref();  // Poll me again right away

// Delayed wake (real async operation)
let waker = cx.waker().clone();
thread::spawn(move || {
    // Do work...
    waker.wake();  // Wake when work is done
});

// Conditional wake (based on events)
if data_available() {
    cx.waker().wake_by_ref();
}
```

### Without the Waker

If we removed the wake call:

```rust
fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
    if !self.polled {
        self.polled = true;
        // cx.waker().wake_by_ref();  // REMOVED!
        Poll::Pending
    } else {
        Poll::Ready("Done")
    }
}
```

**Result:** The program would **hang forever** at the `.await`!

**Why:**
- First poll returns `Pending`
- No wake call means no signal to poll again
- Runtime waits indefinitely for a wake that never comes
- Deadlock! 🔒

## Practical Applications

### 1. Yield Point

Allow other tasks to run:

```rust
struct YieldFuture {
    yielded: bool,
}

impl Future for YieldFuture {
    type Output = ();
    
    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<()> {
        if !self.yielded {
            self.yielded = true;
            cx.waker().wake_by_ref();
            Poll::Pending  // Give other tasks a chance to run
        } else {
            Poll::Ready(())
        }
    }
}

// Usage
async fn cooperative_task() {
    loop {
        // Do some work
        process_items(100);
        
        // Yield to other tasks
        YieldFuture { yielded: false }.await;
    }
}
```

### 2. Multi-Stage Operation

Represent operations that take multiple steps:

```rust
enum Stage {
    Init,
    Processing,
    Complete,
}

struct MultiStageFuture {
    stage: Stage,
}

impl Future for MultiStageFuture {
    type Output = String;
    
    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<String> {
        match self.stage {
            Stage::Init => {
                self.stage = Stage::Processing;
                cx.waker().wake_by_ref();
                Poll::Pending
            }
            Stage::Processing => {
                self.stage = Stage::Complete;
                cx.waker().wake_by_ref();
                Poll::Pending
            }
            Stage::Complete => {
                Poll::Ready("All stages done".to_string())
            }
        }
    }
}
```

### 3. Testing Async Code

Test that code properly handles pending futures:

```rust
#[tokio::test]
async fn test_handles_pending() {
    let delayed = DelayedFuture { polled: false };
    
    // This will test that our code waits properly
    let result = timeout(Duration::from_secs(1), delayed).await;
    
    assert_eq!(result.unwrap(), "Done");
}
```

## Comparison: Different Future Behaviors

| Future Type | First Poll | Second Poll | Third Poll | Use Case |
|------------|-----------|-------------|------------|----------|
| `ReadyFuture` | `Ready` | N/A | N/A | Immediate values |
| `DelayedFuture` | `Pending` | `Ready` | N/A | Yield once |
| `TimerFuture` | `Pending` | `Pending` | `Ready` | Real delays |
| `IoFuture` | `Pending` | `Pending`* | `Ready`* | I/O operations |

*May take many polls depending on when data arrives

## State Transition Diagram

```
    ┌─────────────┐
    │   Created   │
    │ polled=false│
    └──────┬──────┘
           │
           │ .await
           ▼
    ┌─────────────┐
    │ First Poll  │
    │   Pending   │
    └──────┬──────┘
           │
           │ set polled=true
           │ wake_by_ref()
           ▼
    ┌─────────────┐
    │ Scheduled   │
    │ polled=true │
    └──────┬──────┘
           │
           │ runtime polls again
           ▼
    ┌─────────────┐
    │ Second Poll │
    │    Ready    │
    └──────┬──────┘
           │
           │ return "Done"
           ▼
    ┌─────────────┐
    │  Complete   │
    └─────────────┘
```

## Key Takeaways

1. **State Management**: The `polled` boolean tracks whether we've been polled before
2. **Pending First**: Returning `Pending` allows deferring completion to a later poll
3. **Waker is Critical**: `cx.waker().wake_by_ref()` is essential - without it, the future hangs
4. **Cooperative Scheduling**: This pattern allows other tasks to run between polls
5. **Building Block**: This is the foundation for understanding complex async operations

## Common Pitfalls

### ❌ Forgetting to Wake

```rust
// BAD - will hang forever!
if !self.polled {
    self.polled = true;
    // Missing: cx.waker().wake_by_ref();
    Poll::Pending
}
```

### ❌ Forgetting `mut`

```rust
// BAD - can't modify polled
fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
    // Error: cannot assign to self.polled
    self.polled = true;
}
```

### ❌ Returning Ready Multiple Times

```rust
// BAD - futures should only complete once
fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<&'static str> {
    Poll::Ready("Done")  // Always ready - polled field ignored!
}
```

## Summary

This `DelayedFuture` demonstrates the core mechanics of asynchronous programming in Rust:

- **Stateful execution**: Tracking progress across multiple polls
- **Cooperative scheduling**: Returning `Pending` to yield control
- **Wake protocol**: Using wakers to signal readiness
- **Poll-based model**: The foundation of all async operations in Rust

While real async operations involve actual I/O or timers, this simplified version captures the essential pattern that makes Rust's async system work.