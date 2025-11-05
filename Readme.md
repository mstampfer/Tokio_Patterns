# Tokio Patterns
This repository is a collection of Tokio (Rust async runtime) patterns and examples

## Table of Contents
### Part 1: <span style="color:yellow">Basic Operations</span>
#### Section 1. [Manual Tokio Runtime Creation](basic_operations/tokio_main_macro.md)

Instead of using the `#[tokio::main]` macro, manually create a Tokio runtime

#### Section 2. [Multithreaded Runtime](basic_operations/multi_threaded.md)

Configure the runtime to use 2 worker threads

#### Section 3. [Current Thread Runtime vs Multithread Runtime](basic_operations/current_thread_runtime.md)

This code demonstrates how to create a **single-threaded** Tokio runtime using `new_current_thread()` instead of a multi-threaded runtime.

### Part 2: <span style="color:yellow">Spawning</span>

#### Section 1: [Async Function](spawning/async_function.md)

This Rust code demonstrates basic asynchronous task spawning using the Tokio runtime.

### Section 2: [How Arc Shares Vector Data Across Multiple Tasks](spawning/spawning_with_owned_data.md)

This code demonstrates safe shared ownership of data across multiple asynchronous tasks using Arc

### Section 3: [Task Cancellation](spawning/task_cancellation.md)

This code demonstrates how to stop a running asynchronous task before it completes naturally. 

### Section 4: [How Tokio Ensures Data is Send in Spawned Tasks](spawning/send_bound.md)

This code demonstrates Rust's Send trait enforcement for data shared across asynchronous tasks.

### Part 3: <span style="color:yellow">Shared State</span><br>
#### Section 1. [How Arc Shares Immutable Data Across Multiple Tasks](shared_state/arc_sharing_explanation.md)

This code demonstrates reference-counted thread-safe sharing of immutable data using Arc

#### Section 2. [How a Mutex Shares Mutable State](shared_state/mutex_explanation.md)

This code demonstrates safe concurrent access to shared mutable state using Arc and Mutex

#### Section 3. [How RwLock Enables Multiple Concurrent Readers](shared_state/rwlock_explanation.md)

This code demonstrates how RwLock (Read-Write Lock) enables multiple concurrent readers while maintaining exclusive access for writers. 

#### Section 4. [How Semaphores Limit Concurrent Access](shared_state/semaphore_explanation.md)

A semaphore is a synchronization primitive that limits the number of tasks that can access a resource simultaneously.

#### Section 5. [Deadlock Prevention in Concurrent Code](shared_state/deadlock_prevention.md)

A deadlock occurs when two or more tasks are waiting for each other to release resources, creating a circular dependency where none can proceed.

#### Section 6. [How Barriers Work for Task Synchronization](shared_state/barrier_explanation.md) 

A Barrier is a synchronization point where tasks must wait until a specified number of tasks reach that point, then all proceed together.

#### Section 7. [How Notify Works for Signaling Between Tasks](shared_state/notify_explanation.md)
Notify is a simple, lightweight synchronization primitive for signaling between tasks. One task waits for a signal, another task sends it.

#### Section 8. [How Watch Channels Broadcast State Changes](shared_state/watch_channel_explanation.md)

This code demonstrates how a watch channel broadcasts state changes to multiple receivers, where each receiver can observe the latest value.

    
