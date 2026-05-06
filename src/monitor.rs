use std::sync::{Arc, mpsc};
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::thread;
use std::time::{Duration, Instant};

//struct responsible of monitoring the system state (CPU usage, active workers, completed tasks) and printing periodic reports.
pub struct MonitorReport {
    pub sample_count: u64,
    pub cpu_sum: u64,
    pub active_sum: u64,
}

// The Monitor struct holds references to the shared atomic counters for completed tasks, CPU usage, and active workers, 
// as well as a channel sender to report the final statistics when the monitor thread finishes.
impl MonitorReport {
    pub fn avg_cpu(&self) -> f64 {
        self.cpu_sum as f64 / self.sample_count as f64
    }

    pub fn avg_active(&self) -> f64 {
        self.active_sum as f64 / self.sample_count as f64
    }
}

//this is the monitor struct, which will be used to create a monitor thread that will periodically samples the system state and prints it to the console.
pub struct Monitor {
    completed: Arc<AtomicUsize>,
    cpu_usage: Arc<AtomicUsize>,
    active: Arc<AtomicUsize>,
    start: Instant,
    shutdown: Arc<AtomicBool>,
    tx: mpsc::Sender<MonitorReport>,
    num_workers: usize,
}

// The Monitor struct has a method start() that spawns a new thread which continuously samples the CPU usage, active workers,
//  and completed tasks every 10ms until the shutdown flag is set.
impl Monitor {
    pub fn new(
        completed: Arc<AtomicUsize>,
        cpu_usage: Arc<AtomicUsize>,
        active: Arc<AtomicUsize>,
        start: Instant,
        shutdown: Arc<AtomicBool>,
        tx: mpsc::Sender<MonitorReport>,
        num_workers: usize,
    ) -> Self {
        Monitor {
            completed,
            cpu_usage,
            active,
            start,
            shutdown,
            tx,
            num_workers,
        }
    }

    pub fn start(self) {
        //initiates the thread that will monitor the system state and print periodic reports. 
        thread::spawn(move || {
            let mut sample_count = 0;
            let mut cpu_sum = 0;
            let mut active_sum = 0;

            // The monitor thread will run until the shutdown flag is set.
            while !self.shutdown.load(Ordering::Relaxed) {
                let cpu = self.cpu_usage.load(Ordering::Relaxed);
                let active = self.active.load(Ordering::Relaxed);
                let done = self.completed.load(Ordering::Relaxed);

                sample_count += 1;
                cpu_sum += cpu as u64;
                active_sum += active as u64;

                //the elapsed time since the start of the experiment is calculated and printed along with the current CPU usage, active workers, and completed tasks.
                let elapsed = self.start.elapsed().as_millis();
                println!(
                    "[monitor {:>4}ms] active={}/{}  cpu={}%  completed={}",
                    elapsed, active, self.num_workers, cpu, done
                );

                // The monitor thread sleeps for 10ms before taking the next sample.
                thread::sleep(Duration::from_millis(10));
            }

            // When the shutdown flag is set, the monitor thread sends a final report with the average CPU usage and active workers over the sampling period.
            self.tx
                .send(MonitorReport {
                    sample_count,
                    cpu_sum,
                    active_sum,
                })
                .unwrap();
        });
    }
}
