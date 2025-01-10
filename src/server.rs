use std::sync::{Arc, Mutex};
use std::sync::atomic::AtomicBool;
use std::thread::{self, JoinHandle};
use crate::TaskQueue;

pub type SharedQueue = Arc<Mutex<TaskQueue>>;

/// `Scheduler` handles all task scheduling logic. It will update the active
/// task based on the queue priority on a fixed timeout. `Scheduler` contains
/// the `sigterm` field, which is an `AtomicBool` that should be set to `true`
/// when the program is meant to exit.
pub struct Scheduler {
    tasks: SharedQueue,
    sigterm: Arc<AtomicBool>,
}

impl Scheduler {
    /// Creates a new `Scheduler` with the given sigterm and task queue.
    pub fn with_queue(queue: SharedQueue, sigterm: Arc<AtomicBool>) -> Self {
        Self {
            tasks: Arc::clone(&queue),
            sigterm: Arc::clone(&sigterm),
        }
    }
}

/// `Server` handles all communication with clients. This includes waiting for
/// requests, updating shared resources, and sending responses.
pub struct Server {
    tasks: SharedQueue,
    sigterm: Arc<AtomicBool>,
}

impl Server {
    /// Creates a new `Server` with the given sigterm and task queue.
    pub fn with_queue(queue: SharedQueue, sigterm: Arc<AtomicBool>) -> Self {
        Self {
            tasks: Arc::clone(&queue),
            sigterm: Arc::clone(&sigterm),
        }
    }
}
