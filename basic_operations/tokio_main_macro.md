# Manual Tokio Runtime Creation

Instead of using the `#[tokio::main]` macro, you can manually create a Tokio runtime and block on async functions. This gives you more control over the runtime configuration and lifecycle.

## What the `#[tokio::main]` Macro Does

The macro is syntactic sugar that expands your code:

```rust
// This code with the macro:
#[tokio::main]
async fn main() {
    let result = async_function().await;
    println!("Result: {}", result);
}

// Expands to something like this:
fn main() {
    let runtime = tokio::runtime::Runtime::new().unwrap();
    runtime.block_on(async {
        let result = async_function().await;
        println!("Result: {}", result);
    });
}
```

## Basic Manual Runtime Creation

### Simple Example

```rust
fn main() {
    // Create a new Tokio runtime
    let runtime = tokio::runtime::Runtime::new().unwrap();
    
    // Block on an async function
    let result = runtime.block_on(async {
        async_function().await
    });
    
    println!("Result: {}", result);
}

async fn async_function() -> String {
    tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
    "Hello from async!".to_string()
}
```

## Using Runtime Builder for Customization

### Multi-threaded Runtime

```rust
fn main() {
    // Create a runtime with custom configuration
    let runtime = tokio::runtime::Builder::new_multi_thread()
        .worker_threads(4)
        .thread_name("my-custom-thread")
        .enable_all()
        .build()
        .unwrap();
    
    // Block on multiple async operations
    let result = runtime.block_on(async {
        // Spawn multiple tasks
        let task1 = tokio::spawn(async {
            tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
            "Task 1"
        });
        
        let task2 = tokio::spawn(async {
            tokio::time::sleep(tokio::time::Duration::from_millis(200)).await;
            "Task 2"
        });
        
        // Wait for both tasks
        let (res1, res2) = tokio::join!(task1, task2);
        format!("{:?}, {:?}", res1, res2)
    });
    
    println!("Results: {}", result);
}
```

### Single-threaded Runtime

```rust
fn main() {
    // Create a current-thread runtime (single-threaded, lighter weight)
    let runtime = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .unwrap();
    
    runtime.block_on(async {
        println!("Running on single thread");
        async_function().await;
    });
}
```

## Reusing Runtime for Multiple Operations

```rust
fn main() {
    let runtime = tokio::runtime::Runtime::new().unwrap();
    
    // First operation
    let result1 = runtime.block_on(async {
        tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
        "First operation"
    });
    println!("{}", result1);
    
    // Second operation
    let result2 = runtime.block_on(async {
        tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
        "Second operation"
    });
    println!("{}", result2);
}
```

## Using `enter()` for Scoped Context

```rust
fn main() {
    let runtime = tokio::runtime::Runtime::new().unwrap();
    
    // Enter the runtime context without blocking
    let _guard = runtime.enter();
    
    // Now you can spawn tasks without blocking
    runtime.spawn(async {
        println!("Spawned task running");
    });
    
    // Block on the main async operation
    runtime.block_on(async {
        tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
        println!("Main operation complete");
    });
}
```

## Key Methods and Functions

### Runtime Creation

- **`Runtime::new()`** - Creates a default multi-threaded runtime
- **`Builder::new_multi_thread()`** - Creates a customizable multi-threaded runtime
- **`Builder::new_current_thread()`** - Creates a single-threaded runtime

### Runtime Execution

- **`block_on(future)`** - Blocks the current thread until the future completes
- **`spawn(future)`** - Spawns a new async task on the runtime
- **`enter()`** - Enters the runtime context without blocking

### Builder Options

- **`worker_threads(n)`** - Set the number of worker threads
- **`thread_name(name)`** - Set the name prefix for worker threads
- **`enable_all()`** - Enable all runtime features (I/O, time, etc.)
- **`enable_io()`** - Enable only I/O functionality
- **`enable_time()`** - Enable only time functionality

## When to Manually Create a Runtime

**Use manual runtime creation when you need:**

1. **Custom configuration** - Control over thread count, thread names, and other settings
2. **Runtime reuse** - Call `block_on()` multiple times with the same runtime
3. **Testing** - More precise control over runtime lifecycle in tests
4. **Library code** - When you can't use procedural macros
5. **Performance tuning** - Fine-tune runtime for specific workloads
6. **Mixed sync/async code** - Better integration with existing synchronous code

**Use `#[tokio::main]` when:**

1. Writing simple applications
2. You don't need custom runtime configuration
3. You want less boilerplate code

## Common Patterns

### Pattern 1: Global Runtime (Not Recommended for Most Cases)

```rust
use once_cell::sync::Lazy;

static RUNTIME: Lazy<tokio::runtime::Runtime> = Lazy::new(|| {
    tokio::runtime::Runtime::new().unwrap()
});

fn main() {
    let result = RUNTIME.block_on(async {
        // async work
    });
}
```

### Pattern 2: Runtime as Struct Field

```rust
struct App {
    runtime: tokio::runtime::Runtime,
}

impl App {
    fn new() -> Self {
        Self {
            runtime: tokio::runtime::Runtime::new().unwrap(),
        }
    }
    
    fn run(&self) {
        self.runtime.block_on(async {
            // async application logic
        });
    }
}
```

### Pattern 3: Scoped Runtime

```rust
fn main() {
    {
        let runtime = tokio::runtime::Runtime::new().unwrap();
        runtime.block_on(async {
            // async work
        });
        // Runtime is dropped here, cleaning up resources
    }
    
    // Continue with synchronous code
}
```

## Error Handling

```rust
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let runtime = tokio::runtime::Runtime::new()?;
    
    let result = runtime.block_on(async {
        // Your async code that might fail
        Ok::<_, std::io::Error>(())
    })?;
    
    Ok(())
}
```

## Summary

Manual runtime creation gives you complete control over Tokio's async runtime. Use `Runtime::new()` for quick setup, or use `Builder` for customization. The `block_on()` method bridges the gap between sync and async code, allowing you to run async functions in a synchronous context.