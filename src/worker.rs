
use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::thread;
use std::time::Duration;

use crate::queue::TaskQueue;
use crate::task::{TaskKind};
use rand::rngs::StdRng;
use rand::SeedableRng;
// Each worker thread continuously pops tasks from the queue and executes them.
pub fn start_worker(
    worker_id: usize,
    queue: Arc<TaskQueue>,
    completed: Arc<AtomicUsize>,
    cpu_usage: Arc<AtomicUsize>,
    active: Arc<AtomicUsize>,
    //rng: StdRng,
) -> thread::JoinHandle<()> {
    // Each worker runs in its own thread
    thread::spawn(move || {
        let mut _rng = StdRng::seed_from_u64(1000 + worker_id as u64);

        loop {
            // tries  to pop a task from the queue
            let task = queue.pop();
            if task.is_none() {
                break;
            }
            let task = task.unwrap();

            // CPU cost for this task
            let cost = task.kind.cpu_cost();

            // CPU gate: wait until global CPU load allows this task
            loop {
                let result = cpu_usage.fetch_update(
                    Ordering::Relaxed,
                    Ordering::Relaxed,
                    |current| {
                        if current + cost <=100 {
                            Some(current + cost)
                        } else {
                            None
                        }
                    },
                );
                //if successfully updated the cpu usage, break the loop and execute the task
                if result.is_ok() { break; }
               
            }

            // Mark worker as active
            active.fetch_add(1, Ordering::Relaxed);

            // Execute task for fixed 200ms
            match task.kind {
                TaskKind::Cpu => {
                    thread::sleep(Duration::from_millis(200));
                }
                TaskKind::Io => {
                    thread::sleep(Duration::from_millis(200));
                }
            }

            // Release CPU load
            cpu_usage.fetch_sub(cost, Ordering::Relaxed);

            // mark worker as inactive
            active.fetch_sub(1, Ordering::Relaxed);

            // count completed task
            completed.fetch_add(1, Ordering::Relaxed);
            //prints the worker id, task id, task kind and the time taken to execute the task
            println!(
                "  [worker {}] finished task {} ({:?}, 200ms)",
                worker_id, task.id, task.kind
            );
        }
    })
}
