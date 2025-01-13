use crate::{SharedQueue, Task};
use crate::error::SchedulingError;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};

/// `Scheduler` handles all task scheduling logic. It will update the active
/// task based on the queue priority on a fixed timeout.
pub struct Scheduler {
    tasks: SharedQueue,
    active_task: Option<Task>,
}

impl Scheduler {
    /// Creates a new `Scheduler` with the given task queue.
    pub fn with_queue(queue: SharedQueue) -> Self {
        Self {
            tasks: Arc::clone(&queue),
            active_task: None,
        }
    }

    /// Updates the scheduling logic on a timed loop. The `sigterm` parameter
    /// should be set to `true` when the program exits, at which point all data
    /// will be serialized and written to disk.
    pub async fn run(&mut self, sigterm: AtomicBool) -> Result<(), SchedulingError> {
        while !sigterm.load(Ordering::Relaxed) {
            let queue = self.tasks.lock()?;
            if queue.enabled {
                unimplemented!()
            }
            std::thread::sleep(std::time::Duration::from_secs(5));
        }

        Ok(())
    }
}

