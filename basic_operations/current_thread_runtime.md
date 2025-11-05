# Current-Thread Runtime vs Multi-Threaded Runtime

This code demonstrates how to create a **single-threaded** Tokio runtime using `new_current_thread()` instead of a multi-threaded runtime.

## The Code

```rust
use tokio::runtime::Builder;

async fn simple_task() {
    println!("Running on current thread");
}

fn main() {
    // Create a current-thread runtime
    let rt = Builder::new_current_thread()
        .build()
        .unwrap();
    
    rt.block_on(simple_task());
}
```

## Key Difference: `new_current_thread()` vs `new_multi_thread()`

### Current-Thread Runtime (Single-Threaded)

```rust
Builder::new_current_thread()  // ← Single-threaded
    .build()
    .unwrap();
```

**Characteristics:**
- All async tasks run on **one thread** (the thread that calls `block_on`)
- No thread pool is created
- Tasks are executed cooperatively on the current thread
- Lighter weight and lower overhead
- Tasks cannot run in parallel - they're interleaved through cooperative multitasking

### Multi-Threaded Runtime

```rust
Builder::new_multi_thread()  // ← Multi-threaded
    .build()
    .unwrap();
```

**Characteristics:**
- Creates a **thread pool** with multiple worker threads (default: number of CPU cores)
- Tasks can run in **true parallel** across multiple threads
- Higher overhead but better for CPU-bound work
- Tasks are distributed across available worker threads

## Visual Comparison

### Current-Thread Runtime (This Code)

```
Main Thread
├── Runtime created on main thread
├── block_on(simple_task())
│   └── simple_task() executes HERE (same thread)
└── All spawned tasks also run on THIS thread (interleaved)
```

### Multi-Threaded Runtime

```
Main Thread
├── Runtime creates worker thread pool
│   ├── Worker Thread 1
│   ├── Worker Thread 2
│   ├── Worker Thread 3
│   └── Worker Thread 4
├── block_on(simple_task())
│   └── simple_task() executes on one of the worker threads
└── Tasks distributed across all worker threads
```

## Example: Spawning Multiple Tasks

### With Current-Thread Runtime

```rust
fn main() {
    let rt = Builder::new_current_thread()
        .build()
        .unwrap();
    
    rt.block_on(async {
        // These all run on the SAME thread, interleaved
        let task1 = tokio::spawn(async {
            println!("Task 1 on {:?}", std::thread::current().id());
        });
        let task2 = tokio::spawn(async {
            println!("Task 2 on {:?}", std::thread::current().id());
        });
        let task3 = tokio::spawn(async {
            println!("Task 3 on {:?}", std::thread::current().id());
        });
        
        let _ = tokio::join!(task1, task2, task3);
    });
}
```

**Output:**
```
Task 1 on ThreadId(1)
Task 2 on ThreadId(1)  // ← Same thread!
Task 3 on ThreadId(1)  // ← Same thread!
```

### With Multi-Threaded Runtime

```rust
fn main() {
    let rt = Builder::new_multi_thread()
        .worker_threads(4)
        .build()
        .unwrap();
    
    rt.block_on(async {
        // These can run on DIFFERENT threads
        let task1 = tokio::spawn(async {
            println!("Task 1 on {:?}", std::thread::current().id());
        });
        let task2 = tokio::spawn(async {
            println!("Task 2 on {:?}", std::thread::current().id());
        });
        let task3 = tokio::spawn(async {
            println!("Task 3 on {:?}", std::thread::current().id());
        });
        
        let _ = tokio::join!(task1, task2, task3);
    });
}
```

**Output:**
```
Task 1 on ThreadId(2)
Task 2 on ThreadId(3)  // ← Different thread!
Task 3 on ThreadId(4)  // ← Different thread!
```

## When to Use Each

### Use Current-Thread Runtime When:

✅ **I/O-bound operations** - Network requests, file I/O, database queries  
✅ **Single-threaded environments** - WebAssembly, embedded systems  
✅ **Lower overhead needed** - Simpler applications, CLI tools  
✅ **Deterministic behavior** - Testing, debugging  
✅ **No need for parallelism** - Sequential async operations  

**Example use cases:**
- Simple HTTP client making a few requests
- CLI tool reading/writing files
- WebSocket client
- Small scripts with async I/O

### Use Multi-Threaded Runtime When:

✅ **CPU-bound operations** - Computation-heavy tasks  
✅ **High concurrency** - Handling many simultaneous operations  
✅ **Parallel processing** - Need true parallelism across cores  
✅ **Server applications** - Web servers, API servers  
✅ **Maximum throughput** - Processing large volumes of data  

**Example use cases:**
- Web server handling thousands of requests
- Data processing pipelines
- Parallel computation tasks
- High-performance applications

## Performance Characteristics

| Aspect | Current-Thread | Multi-Threaded |
|--------|---------------|----------------|
| **Memory overhead** | Low | Higher |
| **Context switching** | Minimal | More frequent |
| **True parallelism** | No | Yes |
| **Startup time** | Fast | Slower |
| **Best for** | I/O-bound | CPU-bound + I/O-bound |

## Code Comparison Summary

```rust
// Single-threaded (this code)
let rt = Builder::new_current_thread()  // ← Only difference
    .build()
    .unwrap();

// Multi-threaded
let rt = Builder::new_multi_thread()    // ← Only difference
    .build()
    .unwrap();

// Both use the same API
rt.block_on(async_task());
```

## Important Notes

1. **Cooperative multitasking**: Even with current-thread runtime, multiple tasks can appear to run "concurrently" through cooperative scheduling (tasks yield at `.await` points)

2. **Not truly parallel**: Current-thread runtime cannot utilize multiple CPU cores for parallel execution

3. **Blocking operations**: Avoid blocking operations in current-thread runtime as they block the entire runtime (use `tokio::task::spawn_blocking` in multi-threaded runtime instead)

4. **Default behavior**: `Runtime::new()` creates a multi-threaded runtime by default

## Complete Example: Comparing Both

```rust
use tokio::runtime::Builder;
use std::time::Duration;

async fn io_task(id: i32) {
    println!("Task {} starting on {:?}", id, std::thread::current().id());
    tokio::time::sleep(Duration::from_millis(100)).await;
    println!("Task {} finished on {:?}", id, std::thread::current().id());
}

fn main() {
    println!("=== Current-Thread Runtime ===");
    let rt_current = Builder::new_current_thread()
        .build()
        .unwrap();
    
    rt_current.block_on(async {
        let t1 = tokio::spawn(io_task(1));
        let t2 = tokio::spawn(io_task(2));
        let _ = tokio::join!(t1, t2);
    });
    
    println!("\n=== Multi-Thread Runtime ===");
    let rt_multi = Builder::new_multi_thread()
        .worker_threads(2)
        .build()
        .unwrap();
    
    rt_multi.block_on(async {
        let t1 = tokio::spawn(io_task(1));
        let t2 = tokio::spawn(io_task(2));
        let _ = tokio::join!(t1, t2);
    });
}
```

## The Key Takeaway

The **only difference** in this code is `new_current_thread()` instead of `new_multi_thread()`. This single method call determines whether your runtime uses:
- **One thread** (current-thread) - lighter, simpler, no parallelism
- **Multiple threads** (multi-threaded) - heavier, parallel, better for high concurrency

For most I/O-bound async applications, current-thread runtime is sufficient and more efficient!