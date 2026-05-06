//the mods are the different modules that make up the project, each handling a specific aspect of the simulation.
// The main module (main.rs) is responsible for setting up the simulation, generating tasks, and starting the worker and monitor threads.
mod monitor;
mod queue;
mod task;
mod worker;

use monitor::Monitor;
use queue::TaskQueue;
use task::{Task, TaskKind};
use worker::start_worker;

use rand::rngs::StdRng;
use rand::{Rng, SeedableRng};

use std::sync::Arc;
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::sync::mpsc;
use std::thread;
use std::time::{Duration, Instant};

#[derive(Clone)]
struct Config {
    name: &'static str,
    num_workers: usize,
    tasks_per_worker: usize,
    io_probability: f64,
}
// Baseline config: 70% IO, 30% CPU tasks
impl Config {
    fn baseline() -> Self {
        Config {
            name: "Baseline (70% IO / 30% CPU)",
            num_workers: 8,
            tasks_per_worker: 125, // 8 * 125 = 1000
            io_probability: 0.70,
        }
    }
// Optimized config: 70% IO, 30% CPU tasks (same as baseline for this experiment)
    fn optimized() -> Self {
        Config {
            name: "Optimized (70% IO / 30% CPU)",
            num_workers: 8,
            tasks_per_worker: 125,
            io_probability: 0.70,
        }
    }
}

fn run_simulation(config: &Config) {
    println!("\n##### RUNNING EXPERIMENT: {} #####", config.name);

    // Start experiment timer
    let start = Instant::now();

    // Shared atomic counters
    let completed = Arc::new(AtomicUsize::new(0));
    let cpu_usage = Arc::new(AtomicUsize::new(0));
    let active_workers = Arc::new(AtomicUsize::new(0));
    let shutdown = Arc::new(AtomicBool::new(false));

    // Counters for IO and CPU tasks
    let io_count = Arc::new(AtomicUsize::new(0));
    let cpu_count = Arc::new(AtomicUsize::new(0));

    // Channel for monitor report
    let (report_tx, report_rx) = mpsc::channel();

    // Start monitor thread
    let mon = Monitor::new(
        Arc::clone(&completed),
        Arc::clone(&cpu_usage),
        Arc::clone(&active_workers),
        start,
        Arc::clone(&shutdown),
        report_tx,
        config.num_workers,
    );
    mon.start();

    // Create FIFO task queue
    let queue = TaskQueue::new();

    //generate tasks every 20ms
    let total_tasks = config.num_workers * config.tasks_per_worker;
    let mut rng = StdRng::from_entropy();

    for id in 0..total_tasks {
        // Randomly choose IO or CPU task
        let roll: u32 = rng.gen_range(0..100);
        let threshold = (config.io_probability * 100.0) as u32;
        let is_io = roll < threshold;

        let kind = if is_io { TaskKind::Io } else { TaskKind::Cpu };

        // Count IO/CPU tasks
        if is_io {
            io_count.fetch_add(1, Ordering::Relaxed);
        } else {
            cpu_count.fetch_add(1, Ordering::Relaxed);
        }

        // Push task into queue
        queue.push(Task {
            id,
            kind,
            arrival_ms: start.elapsed().as_millis(),
        });

        println!("Task {} arrived at {} ms", id, start.elapsed().as_millis());

        thread::sleep(Duration::from_millis(20));
    }

    // Close queue so workers know no more tasks will arrive
    queue.close();

    // Spawn worker threads
    let mut handles = Vec::new();

    for worker_id in 0..config.num_workers {
        let handle = start_worker(
            worker_id,
            Arc::clone(&queue),
            Arc::clone(&completed),
            Arc::clone(&cpu_usage),
            Arc::clone(&active_workers),
        );

        handles.push(handle);
    }

    // Wait for all workers to finish
    for h in handles {
        h.join().unwrap();
    }

    // Tell monitor to stop
    shutdown.store(true, Ordering::Relaxed);

    // Receive final report
    let report = report_rx.recv().unwrap();

    // Print summary from the monitor report and final stats
    println!("\n####### FINAL REPORT: {} #######", config.name);
    println!("Total time:       {} ms", start.elapsed().as_millis());
    println!("Tasks completed:  {}", completed.load(Ordering::Relaxed));
    println!("IO tasks:         {}", io_count.load(Ordering::Relaxed));
    println!("CPU tasks:        {}", cpu_count.load(Ordering::Relaxed));
    println!("Average CPU:      {:.1}%", report.avg_cpu());
    println!("Average active:   {:.2} / {} workers", report.avg_active(), config.num_workers);
    println!("####################################\n");
}

fn main() {
    run_simulation(&Config::baseline());
    run_simulation(&Config::optimized());
}
