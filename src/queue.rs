use std::sync::{Arc, Mutex, Condvar};
use crate::task::Task;
//is the thread queue implementation used by the worker threads to get tasks to execute. 
// It uses a Mutex and Condvar to manage access to the queue 
// and allow workers to wait for tasks when the queue is empty. 
// The queue supports pushing new tasks, popping tasks for execution, 
// and closing the queue to signal that no more tasks will be added.
pub struct TaskQueue {
    inner: Arc<(Mutex<QueueState>, Condvar)>,
}
// Internal state of the TaskQueue, protected by a Mutex
struct QueueState {
    tasks: Vec<Task>,
    closed: bool,
}
// Implementation of TaskQueue with methods to create a new queue, push tasks, pop tasks, and close the queue.
impl TaskQueue {
    pub fn new() -> Arc<Self> {
        Arc::new(TaskQueue {
            inner: Arc::new((Mutex::new(QueueState {
                tasks: Vec::new(),
                closed: false,
            }), Condvar::new())),
        })
    }

    pub fn push(&self, task: Task) {
        let (lock, cvar) = &*self.inner;
        let mut state = lock.lock().unwrap();
        state.tasks.push(task);
        cvar.notify_one();
    }

    pub fn pop(&self) -> Option<Task> {
        let (lock, cvar) = &*self.inner;
        let mut state = lock.lock().unwrap();

        loop {
            if let Some(task) = state.tasks.pop() {
                return Some(task);
            }
            if state.closed {
                return None;
            }
            state = cvar.wait(state).unwrap();
        }
    }

    pub fn close(&self) {
        let (lock, cvar) = &*self.inner;
        let mut state = lock.lock().unwrap();
        state.closed = true;
        cvar.notify_all();
    }
}
