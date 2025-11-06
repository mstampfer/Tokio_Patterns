# Building a Simple Future Executor with Custom Waker

## Complete Code

```rust
use std::future::Future;
use std::pin::Pin;
use std::task::{Context, Poll, Waker, RawWaker, RawWakerVTable};

fn dummy_raw_waker() -> RawWaker {
    fn no_op(_: *const ()) {}
    fn clone(_: *const ()) -> RawWaker {
        dummy_raw_waker()
    }
    
    let vtable = &RawWakerVTable::new(clone, no_op, no_op, no_op);
    RawWaker::new(std::ptr::null(), vtable)
}

fn dummy_waker() -> Waker {
    unsafe { Waker::from_raw(dummy_raw_waker()) }
}

fn block_on<F: Future>(mut future: F) -> F::Output {
    // Use pin_mut! macro or std::pin::pin! to safely pin on the stack
    let mut future = std::pin::pin!(future);
    let waker = dummy_waker();
    let mut context = Context::from_waker(&waker);
    
    loop {
        match future.as_mut().poll(&mut context) {
            Poll::Ready(value) => return value,
            Poll::Pending => {
                // In a real executor, we'd wait for the waker to be called
                // For this simple executor, we just keep polling
            }
        }
    }
}

async fn simple() -> i32 {
    42
}

fn main() {
    let result = block_on(simple());
    println!("Result: {}", result);
}
```

## Output

```
Result: 42
```

## Understanding the Custom Executor

### What is an Executor?

An executor is the runtime system that drives futures to completion. It's responsible for:

1. **Polling futures** - Calling the `poll` method repeatedly
2. **Managing wakers** - Providing a way for futures to signal when they're ready
3. **Scheduling** - Deciding when and in what order to poll futures
4. **Completion** - Extracting the final value when a future is ready

**Key concept:** Futures are lazy and do nothing until an executor polls them.

### The Three Core Components

Our simple executor has three main parts:

```
┌─────────────────┐
│   RawWaker      │ ← Low-level waker implementation
└────────┬────────┘
         │
         ▼
┌─────────────────┐
│     Waker       │ ← Safe wrapper around RawWaker
└────────┬────────┘
         │
         ▼
┌─────────────────┐
│   block_on      │ ← The executor that polls the future
└─────────────────┘
```

## Breaking Down the Code

### Part 1: Creating a RawWaker

```rust
fn dummy_raw_waker() -> RawWaker {
    fn no_op(_: *const ()) {}
    fn clone(_: *const ()) -> RawWaker {
        dummy_raw_waker()
    }
    
    let vtable = &RawWakerVTable::new(clone, no_op, no_op, no_op);
    RawWaker::new(std::ptr::null(), vtable)
}
```

#### What is a RawWaker?

A `RawWaker` is the low-level building block for wakers. It consists of:
- A data pointer (`*const ()`)
- A vtable with function pointers for clone, wake, wake_by_ref, and drop operations

**Structure:**
```rust
pub struct RawWaker {
    data: *const (),        // Pointer to waker's data
    vtable: &'static RawWakerVTable,  // Function pointers
}

pub struct RawWakerVTable {
    clone: fn(*const ()) -> RawWaker,      // Clone the waker
    wake: fn(*const ()),                   // Consume and wake
    wake_by_ref: fn(*const ()),            // Wake without consuming
    drop: fn(*const ()),                   // Drop the waker
}
```

#### Our Implementation

**The no-op function:**
```rust
fn no_op(_: *const ()) {}
```
- Does nothing
- Used for `wake`, `wake_by_ref`, and `drop`
- In a real executor, these would trigger scheduling

**The clone function:**
```rust
fn clone(_: *const ()) -> RawWaker {
    dummy_raw_waker()
}
```
- Creates a new `RawWaker` when cloning
- Ignores the data pointer (we don't have any data)

**Creating the vtable:**
```rust
let vtable = &RawWakerVTable::new(clone, no_op, no_op, no_op);
```
- `clone`: Our clone function
- `wake`: no-op (doesn't schedule anything)
- `wake_by_ref`: no-op (doesn't schedule anything)
- `drop`: no-op (no cleanup needed)

**Creating the RawWaker:**
```rust
RawWaker::new(std::ptr::null(), vtable)
```
- Data pointer: `null` (we don't need any data)
- Vtable: Our function pointer table

### Part 2: Creating a Waker

```rust
fn dummy_waker() -> Waker {
    unsafe { Waker::from_raw(dummy_raw_waker()) }
}
```

**What is a Waker?**
- Safe wrapper around `RawWaker`
- Used by futures to signal they're ready to make progress
- The `Context` passed to `poll` contains a `Waker`

**Why unsafe?**
- `Waker::from_raw` is unsafe because it trusts that the `RawWaker` is valid
- The caller must ensure the vtable functions are safe to call
- Our implementation is safe (functions do nothing or create new wakers)

**Safety guarantee:**
Our `dummy_waker` is safe because:
- The vtable functions are all safe (no-op or clone)
- No shared mutable state
- No invalid pointer dereferences

### Part 3: The block_on Executor

```rust
fn block_on<F: Future>(mut future: F) -> F::Output {
    let mut future = std::pin::pin!(future);
    let waker = dummy_waker();
    let mut context = Context::from_waker(&waker);
    
    loop {
        match future.as_mut().poll(&mut context) {
            Poll::Ready(value) => return value,
            Poll::Pending => {
                // In a real executor, we'd wait for the waker to be called
                // For this simple executor, we just keep polling
            }
        }
    }
}
```

Let's break this down step by step:

#### Step 1: Pin the Future

```rust
let mut future = std::pin::pin!(future);
```

**Why pin?**
- The `poll` method requires `Pin<&mut Self>`
- Futures can be self-referential and must not move in memory
- `std::pin::pin!` safely pins the future on the stack

**What `pin!` does:**
```rust
// Conceptually:
let mut future = future;  // Take ownership
let future = &mut future; // Create mutable reference
let future = Pin::new_unchecked(future); // Pin it (but done safely)
```

The macro ensures the original `future` binding is shadowed and can't be accessed, preventing moves.

#### Step 2: Create Waker and Context

```rust
let waker = dummy_waker();
let mut context = Context::from_waker(&waker);
```

**Context:**
- Contains a `Waker` reference
- Passed to `poll` so the future can wake itself
- Allows futures to signal "poll me again"

**Structure:**
```rust
pub struct Context<'a> {
    waker: &'a Waker,
    // ... other fields
}
```

#### Step 3: Poll Loop

```rust
loop {
    match future.as_mut().poll(&mut context) {
        Poll::Ready(value) => return value,
        Poll::Pending => {
            // Keep polling
        }
    }
}
```

**The polling cycle:**

1. **Call `poll`**: `future.as_mut().poll(&mut context)`
   - `as_mut()` converts `Pin<&mut F>` to `Pin<&mut F>` (maintains pin)
   - Passes the context with the waker
   
2. **Check result**:
   - `Poll::Ready(value)`: Future is done, return the value
   - `Poll::Pending`: Future needs more time, keep polling

3. **Repeat**: If `Pending`, loop back and poll again

**Why `as_mut()`?**
- We have `Pin<&mut F>` and need to call `poll` multiple times
- `as_mut()` reborrows the pinned reference
- Allows us to poll again in the next iteration

## Execution Flow Diagram

### Visual Timeline

```
Time    State                              Action
─────────────────────────────────────────────────────────────
T0      main() starts                      
        └─> call block_on(simple())        
                                            
T1      block_on starts                    
        ├─> Pin future on stack            
        ├─> Create dummy waker             
        └─> Create context                 
                                            
T2      Enter poll loop                    
        └─> Call future.poll(&mut context) 
                                            
T3      Inside simple() future             
        └─> Returns Poll::Ready(42)        
                                            
T4      Match on Poll::Ready(42)           
        └─> Return 42 from block_on        
                                            
T5      Back in main()                     
        └─> Print "Result: 42"             
```

### Memory Layout

```
Stack Frame (block_on):
┌─────────────────────────┐
│ future: Pin<&mut F>     │───┐
│ (pinned on stack)       │   │
├─────────────────────────┤   │
│ waker: Waker            │   │
│ ├─ RawWaker             │   │
│ │  ├─ data: null        │   │
│ │  └─ vtable: &VTable   │   │
│ └─ ...                  │   │
├─────────────────────────┤   │
│ context: Context        │   │
│ └─ waker: &Waker        │───┘
└─────────────────────────┘

VTable (static):
┌─────────────────────────┐
│ clone: fn(...)          │
│ wake: fn(...)           │
│ wake_by_ref: fn(...)    │
│ drop: fn(...)           │
└─────────────────────────┘
```

## How Polling Works

### The poll Method Signature

```rust
fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output>
```

**Parameters:**
- `self: Pin<&mut Self>`: Pinned mutable reference to the future
- `cx: &mut Context<'_>`: Context containing the waker

**Return:**
- `Poll::Ready(T)`: Future completed with value `T`
- `Poll::Pending`: Future not ready, will notify via waker

### What Happens During poll

For our `simple()` async function:

```rust
async fn simple() -> i32 {
    42
}
```

The compiler generates something like:

```rust
struct SimpleFuture {
    state: State,
}

enum State {
    Start,
    Done,
}

impl Future for SimpleFuture {
    type Output = i32;
    
    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<i32> {
        match self.state {
            State::Start => {
                self.state = State::Done;
                Poll::Ready(42)
            }
            State::Done => {
                panic!("Future polled after completion")
            }
        }
    }
}
```

**First poll:**
1. State is `Start`
2. Transitions to `Done`
3. Returns `Poll::Ready(42)`
4. Executor receives the value and returns it

## Comparison: Simple vs Real Executors

### Our Simple Executor (Busy-Wait)

```rust
loop {
    match future.poll(&mut context) {
        Poll::Ready(value) => return value,
        Poll::Pending => {
            // Just keep polling immediately
        }
    }
}
```

**Characteristics:**
- ✅ Simple to understand
- ✅ Works for immediately-ready futures
- ❌ Wastes CPU cycles (busy-waiting)
- ❌ Can't handle actual async operations efficiently
- ❌ No multi-tasking support

### Real Executor (Event-Driven)

```rust
loop {
    match future.poll(&mut context) {
        Poll::Ready(value) => return value,
        Poll::Pending => {
            // Sleep until waker is called
            // (via epoll/kqueue or other event mechanism)
            parker.park();  // Block thread until woken
        }
    }
}
```

**Characteristics:**
- ✅ Efficient - sleeps when waiting
- ✅ Handles real async I/O
- ✅ Supports multiple concurrent tasks
- ❌ More complex implementation

## What a Real Waker Does

### Our Dummy Waker

```rust
fn no_op(_: *const ()) {}  // Does nothing!
```

When a future calls `cx.waker().wake()`, nothing happens. We just keep polling anyway.

### A Real Waker (Conceptual)

```rust
fn wake(data: *const ()) {
    // Get the task from the data pointer
    let task = unsafe { &*(data as *const Task) };
    
    // Add task back to the run queue
    EXECUTOR.schedule(task);
    
    // Wake up the executor thread if sleeping
    EXECUTOR.notify();
}
```

**What it does:**
1. Retrieves the task information
2. Puts the task back in the executor's queue
3. Wakes up the executor if it's sleeping
4. Executor will poll the task again

### How Futures Use Wakers

```rust
impl Future for TimerFuture {
    type Output = ();
    
    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<()> {
        if self.is_time_reached() {
            Poll::Ready(())
        } else {
            // Register the waker to be called when timer expires
            register_timer_callback(cx.waker().clone());
            Poll::Pending
        }
    }
}
```

**Flow:**
1. Future checks if it's ready
2. Not ready yet, so stores the waker
3. Returns `Poll::Pending`
4. Later, when timer expires, calls `waker.wake()`
5. Executor gets notified and polls the future again
6. This time, future is ready and returns `Poll::Ready(())`

## Practical Example: Real Async Function

### With Actual Async Operation

```rust
use std::future::Future;
use std::pin::Pin;
use std::task::{Context, Poll};
use std::time::{Duration, Instant};

struct DelayFuture {
    deadline: Instant,
}

impl Future for DelayFuture {
    type Output = ();
    
    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<()> {
        if Instant::now() >= self.deadline {
            Poll::Ready(())
        } else {
            // In reality, we'd register the waker with a timer
            // Our simple executor just keeps polling
            cx.waker().wake_by_ref();  // Wake ourselves immediately
            Poll::Pending
        }
    }
}

async fn delay_and_return() -> i32 {
    // Wait for 100ms
    DelayFuture {
        deadline: Instant::now() + Duration::from_millis(100),
    }.await;
    
    42
}

fn main() {
    let start = Instant::now();
    let result = block_on(delay_and_return());
    let elapsed = start.elapsed();
    
    println!("Result: {} after {:?}", result, elapsed);
}
```

**With our simple executor:**
- Keeps polling in a tight loop
- Wastes CPU but eventually completes
- Not efficient but demonstrates the concept

**With a real executor (like Tokio):**
- Registers the waker with an actual timer
- Thread sleeps until timer expires
- Waker wakes the thread, future is polled and completes
- Efficient and correct

## Advanced: Multiple Polls

### Example with Delayed Future

```rust
struct CountdownFuture {
    count: u32,
}

impl Future for CountdownFuture {
    type Output = ();
    
    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<()> {
        println!("Poll #{}", self.count);
        
        if self.count == 0 {
            Poll::Ready(())
        } else {
            self.count -= 1;
            cx.waker().wake_by_ref();  // Wake immediately for next poll
            Poll::Pending
        }
    }
}

fn main() {
    let future = CountdownFuture { count: 3 };
    block_on(future);
}
```

**Output:**
```
Poll #3
Poll #2
Poll #1
Poll #0
```

**What happens:**
1. First poll: count=3, returns Pending, wakes self
2. Second poll: count=2, returns Pending, wakes self
3. Third poll: count=1, returns Pending, wakes self
4. Fourth poll: count=0, returns Ready
5. block_on returns

## Why Wakers Matter

### Without Wakers (Impossible!)

Imagine trying to implement async without wakers:

```rust
loop {
    match future.poll() {  // No context parameter!
        Poll::Ready(value) => return value,
        Poll::Pending => {
            // How do we know when to poll again?
            // Sleep for how long?
            // ???
        }
    }
}
```

**Problem:** No way for the future to tell us when it's ready!

### With Wakers (Solves the Problem!)

```rust
loop {
    match future.poll(&mut context) {  // Context has waker!
        Poll::Ready(value) => return value,
        Poll::Pending => {
            // Future will call waker.wake() when ready
            // Executor can sleep until then
            wait_for_wake();
        }
    }
}
```

**Solution:** Future can signal readiness via the waker!

## The Waker Contract

### What Futures Promise

When a future returns `Poll::Pending`:
1. It has stored the waker (usually by cloning it)
2. It will call `waker.wake()` when it makes progress
3. The executor can safely wait until the wake call

### What Executors Promise

When creating a waker:
1. The waker will schedule the future to be polled again
2. Calling `wake()` is safe and won't cause issues
3. The waker can be cloned and called from any thread

## Limitations of Our Simple Executor

### What It Can't Do

1. **No actual waiting**
   ```rust
   // This won't work efficiently:
   async fn sleep_for_real() {
       tokio::time::sleep(Duration::from_secs(1)).await;
   }
   // Our executor will busy-wait for 1 second!
   ```

2. **No concurrency**
   ```rust
   // Can only run one future at a time:
   block_on(future1);  // Must complete before...
   block_on(future2);  // ...this one starts
   ```

3. **No I/O operations**
   ```rust
   // Won't work properly:
   async fn read_file() {
       tokio::fs::read_to_string("file.txt").await
   }
   // Needs a real event loop!
   ```

### What It Can Do

1. **Run simple futures**
   ```rust
   async fn compute() -> i32 {
       42
   }
   block_on(compute());  // Works!
   ```

2. **Demonstrate concepts**
   - Shows how polling works
   - Shows how wakers are passed
   - Educational value

## Real Executor Examples

### Tokio's Executor (Conceptual)

```rust
// Simplified Tokio executor logic:
struct Executor {
    tasks: VecDeque<Task>,
    parker: Parker,
}

impl Executor {
    fn run(&mut self) {
        loop {
            while let Some(task) = self.tasks.pop_front() {
                let waker = create_waker_for_task(&task);
                let context = Context::from_waker(&waker);
                
                match task.future.poll(&context) {
                    Poll::Ready(value) => {
                        // Task complete
                    }
                    Poll::Pending => {
                        // Task will wake us when ready
                    }
                }
            }
            
            // No more tasks, sleep until woken
            self.parker.park();
        }
    }
}
```

### Creating a Real Waker

```rust
fn create_waker_for_task(task: &Task) -> Waker {
    let task_ptr = task as *const Task;
    
    fn clone(data: *const ()) -> RawWaker {
        // Clone the task reference
        let task = data as *const Task;
        RawWaker::new(task as *const (), &VTABLE)
    }
    
    fn wake(data: *const ()) {
        // Schedule the task
        let task = unsafe { &*(data as *const Task) };
        EXECUTOR.schedule(task);
    }
    
    // Similar for wake_by_ref and drop...
    
    let raw_waker = RawWaker::new(task_ptr as *const (), &VTABLE);
    unsafe { Waker::from_raw(raw_waker) }
}
```

## Key Takeaways

1. **Executors drive futures** - They repeatedly call `poll` until completion

2. **Wakers enable efficiency** - Allow futures to signal when they're ready

3. **Pinning is required** - Futures must be pinned before polling

4. **Context provides access** - The waker is passed via `Context`

5. **RawWaker is low-level** - Building block for implementing wakers

6. **Simple executors teach concepts** - But aren't suitable for production

7. **Real executors are complex** - Handle I/O, threading, and scheduling

## Summary

This code demonstrates the fundamental mechanics of Rust's async runtime system:

1. **RawWaker and Waker**: The mechanism futures use to signal readiness
2. **block_on executor**: A simple polling loop that drives futures to completion
3. **Pinning**: Required for safely polling futures
4. **The poll loop**: Keep polling until the future returns `Poll::Ready`

While our simple executor uses a dummy waker that does nothing (leading to busy-waiting), it clearly illustrates the core concepts:

- Futures are polled repeatedly by an executor
- Wakers provide a way for futures to signal readiness
- The executor creates a context with a waker and passes it to `poll`
- The future returns either `Ready` (done) or `Pending` (not yet)

Real executors like Tokio build on these same concepts but add:
- Efficient sleeping and waking using OS primitives
- Task scheduling and work-stealing
- I/O event integration (epoll/kqueue)
- Multi-threaded execution

Understanding this simple executor is the foundation for understanding how all async Rust works under the hood!