# Rust Multi‑Threaded Concurrency Scheduling Simulation Project

This project simulates a multi‑threaded scheduling system in Rust.  
It models how tasks arrive over time, how worker threads compete for CPU capacity, and how a monitor thread tracks system performance in real time.

The simulation runs **two experiments**:

### **1. Baseline (FIFO Scheduling)**
- Tasks are processed in simple FIFO order.
- Represents normal system behavior.
- Uses a 70% IO / 30% CPU workload.

### **2. Stressload (Optimized Scheduling)**
- Uses the same workload ratio but applies more aggressive scheduling pressure.
- Creates higher CPU contention and worker blocking.
- Behaves like a stress test of the system.


## **Project Structure**

### `task.rs`
Defines the `Task` struct and `TaskKind` (CPU or IO).  
Each task has an ID, type, and arrival timestamp.

### `queue.rs`
Implements a thread‑safe FIFO queue using `Mutex` + `Condvar`.  
Workers block when the queue is empty and exit when the queue is closed.

### `worker.rs`
Defines worker thread behavior:
- pops tasks from the queue  
- waits for CPU capacity  
- executes tasks for 200ms  
- updates shared counters  

### `monitor.rs`
Runs a dedicated monitor thread that samples:
- CPU usage  
- active workers  
- completed tasks  

It prints real‑time system stats every 10ms and sends a final report.

### `main.rs`
Orchestrates the entire experiment:
- creates tasks every 20ms  
- starts the monitor  
- spawns workers  
- runs Baseline and Stressload simulations  
- prints final performance summaries  
