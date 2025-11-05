# How Arc Shares Vector Data Across Multiple Tasks

This code demonstrates **safe shared ownership** of data across multiple asynchronous tasks using `Arc` (Atomic Reference Counting). Here's how it works:

## Code Example

```rust
use std::sync::Arc;
#[tokio::main]
async fn main() {
    let data = Arc::new(vec![1, 2, 3, 4, 5]);
    
    let mut handles = vec![];
    
    for i in 0..3 {
        let data_clone = Arc::clone(&data);
        let handle = tokio::spawn(async move {
            println!("Task {} sees data: {:?}", i, data_clone);
        });
        handles.push(handle);
    }
    
    for handle in handles {
        handle.await.unwrap();
    }
}
```

## The Problem It Solves

When you spawn multiple tasks, each task needs its own data. You can't just pass the same vector to multiple tasks because Rust's ownership rules don't allow multiple owners. You also can't move the data into one task because then the other tasks can't access it.

## The Solution: Arc

**`Arc<T>`** (Atomic Reference Counted) is a smart pointer that allows multiple owners of the same data through reference counting.

## Breaking Down the Code

1. **`let data = Arc::new(vec![1, 2, 3, 4, 5]);`**
   - Wraps the vector in an `Arc`, creating a reference-counted pointer to heap-allocated data
   - The data is now shareable across multiple owners

2. **`let data_clone = Arc::clone(&data);`**
   - Creates a new `Arc` pointer to the *same* underlying data (not a deep copy of the vector!)
   - Increments the reference count atomically (thread-safe)
   - This is cheap - it only clones the pointer, not the actual vector data

3. **`async move { ... }`**
   - The `move` keyword transfers ownership of `data_clone` into the task's closure
   - Each task gets its own `Arc` pointing to the shared vector
   - The original `data` in the main function still exists and keeps the data alive

4. **Reference Counting in Action:**
   - Initially: reference count = 1 (just `data`)
   - After 3 clones: reference count = 4 (1 original + 3 clones)
   - As tasks complete and drop their `Arc`, the count decreases
   - When count reaches 0, the vector is deallocated

## Why This Works

- **Thread-safe:** `Arc` uses atomic operations for reference counting, making it safe across threads/tasks
- **No data races:** The vector is only read (not modified), so multiple tasks can safely access it simultaneously
- **Efficient:** Only the pointer is cloned, not the 5-element vector itself

## Visual Representation

```
Main thread:           Task 0:              Task 1:              Task 2:
data (Arc) ──┐         data_clone (Arc)     data_clone (Arc)     data_clone (Arc)
             │              │                    │                    │
             └──────────────┴────────────────────┴────────────────────┘
                                        │
                                   [1, 2, 3, 4, 5]
                              (single vector in memory)
```

All `Arc` pointers point to the same vector data in memory, but each task owns its own `Arc` handle to that data.

