//this task.rs only contains the definition of Task and TaskKind, which are used by the worker and main modules to create and execute tasks.
// It also includes a method to calculate the CPU cost of each task type.
use std::time::Instant;

#[derive(Clone, Copy, Debug)]
pub enum TaskKind {
    Cpu,
    Io,
}
//returns the CPU cost of the task based on its type (35 for CPU tasks, 10 for IO tasks)
impl TaskKind {
    pub fn cpu_cost(&self) -> usize {
        match self {
            TaskKind::Cpu => 35,
            TaskKind::Io => 10,
        }
    }
}
//struct will be push to the queue, contains the task id, its type (CPU or IO),
//  and the time it was created (arrival_ms)
#[derive(Clone, Debug)]
pub struct Task {
    pub id: usize,
    pub kind: TaskKind,
    pub arrival_ms: u128,
}
