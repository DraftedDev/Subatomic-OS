use crossbeam_queue::SegQueue;

/// A global kernel scheduler instance.
pub static KERNEL_SCHEDULER: KernelScheduler = KernelScheduler::new();

/// A kernel task scheduler.
pub struct KernelScheduler {
    tasks: SegQueue<fn() -> bool>,
}

impl KernelScheduler {
    /// Creates a new kernel scheduler.
    pub const fn new() -> Self {
        Self {
            tasks: SegQueue::new(),
        }
    }

    /// Spawns a task in the scheduler.
    pub fn spawn(&self, task: fn() -> bool) {
        self.tasks.push(task);
    }

    /// Runs the scheduler, executing all tasks in the queue.
    pub fn run(&self) {
        while let Some(task) = self.tasks.pop() {
            let should_repeat = (task)();

            if should_repeat {
                self.tasks.push(task);
            }
        }
    }
}
