use crate::{SharedQueue, Task};
use crate::error::SchedulingError;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::thread::sleep;
use std::time::Duration;
use chrono::TimeDelta;
use piglog::{info, error};

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
            let mut queue = self.tasks.lock()?;

            // if the queue is disabled, skip the iteration.
            if queue.enabled {
                self.active_task = queue.select();

                if let Some(task) = self.active_task.as_mut() {
                    info!("Active task: {} (ID: {})", task.title, task.id);

                    // Tasks are marked as completed from outside of this scope, so
                    // we have to check for it on every loop.
                    if task.completed {
                        queue.delete(task.id)?;
                        queue.add_completed(task.clone());
                    } else {
                        match task.duration.checked_sub(&TimeDelta::seconds(5)) {
                            Some(duration) => task.duration = duration,
                            None => {
                                error!("Task duration overflowed! Something is seriously wrong.");
                            }
                        }
                    }
                } else {
                    info!("No active task.");
                }
            }

            drop(queue);

            sleep(Duration::from_secs(5));
        }

        Ok(())
    }
}
