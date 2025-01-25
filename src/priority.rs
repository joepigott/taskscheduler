use crate::Task;
use serde::{Deserialize, Serialize};

/// A struct implementing the `Priority` trait can be assigned to a `TaskQueue`
/// to define the method for selecting tasks. The important method is
/// `select()` which defines the actual method of selection. To work properly,
/// the `select()` method should **ignore tasks that are flagged as complete**.
///
/// ## Example: `FIFO`
///
/// ```rust
/// pub struct FIFO {}
///
/// impl Priority for FIFO {
///     fn select(&self, queue: &[Task]) -> Option<Task> {
///         queue.first().cloned()
///     }
/// }
/// ```
///
/// ## `clone_box()`
///
/// The `clone_box()` method is required to satisfy the trait bounds for
/// trait object serialization and deserialization. The following
/// implementation will work just fine:
/// ```rust
/// fn clone_box(&self) -> Box<dyn Priority> {
///     Box::new(self.clone())
/// }
/// ```
#[typetag::serde(tag = "type")]
pub trait Priority: Send + Sync {
    fn select(&self, queue: &[Task]) -> Option<Task>;
    fn string(&self) -> String;
    fn clone_box(&self) -> Box<dyn Priority>;
}

/// Schedules tasks in the order they were added to the queue.
#[derive(Clone, Serialize, Deserialize)]
pub struct FIFO;

#[typetag::serde]
impl Priority for FIFO {
    fn select(&self, queue: &[Task]) -> Option<Task> {
        queue.iter().find(|t| !t.completed).cloned()
    }

    fn string(&self) -> String {
        "FIFO".to_string()
    }

    fn clone_box(&self) -> Box<dyn Priority> {
        Box::new(self.clone())
    }
}

/// Schedules tasks in the order they are due.
#[derive(Clone, Serialize, Deserialize)]
pub struct Deadline;

#[typetag::serde]
impl Priority for Deadline {
    fn select(&self, queue: &[Task]) -> Option<Task> {
        queue.iter().min_by_key(|t| t.deadline).cloned()
    }

    fn string(&self) -> String {
        "Deadline".to_string()
    }

    fn clone_box(&self) -> Box<dyn Priority> {
        Box::new(self.clone())
    }
}

/// Schedules tasks in order of increasing duration: short tasks are scheduled
/// ahead of long tasks.
#[derive(Clone, Serialize, Deserialize)]
pub struct Shortest {}

#[typetag::serde]
impl Priority for Shortest {
    fn select(&self, queue: &[Task]) -> Option<Task> {
        queue.iter().min_by_key(|t| t.duration).cloned()
    }

    fn string(&self) -> String {
        "Shortest Duration".to_string()
    }

    fn clone_box(&self) -> Box<dyn Priority> {
        Box::new(self.clone())
    }
}

/// Schedules tasks in order of decreasing duration: long tasks are scheduled
/// ahead of short tasks.
#[derive(Clone, Serialize, Deserialize)]
pub struct Longest {}

#[typetag::serde]
impl Priority for Longest {
    fn select(&self, queue: &[Task]) -> Option<Task> {
        queue.iter().max_by_key(|t| t.duration).cloned()
    }

    fn string(&self) -> String {
        "Longest Duration".to_string()
    }

    fn clone_box(&self) -> Box<dyn Priority> {
        Box::new(self.clone())
    }
}

/// Schedules tasks in order of decreasing priority: higher priority tasks are
/// scheduled ahead of lower priority tasks.
#[derive(Clone, Serialize, Deserialize)]
pub struct HighestPriority {}

#[typetag::serde]
impl Priority for HighestPriority {
    fn select(&self, queue: &[Task]) -> Option<Task> {
        queue.iter().min_by_key(|t| t.priority).cloned()
    }

    fn string(&self) -> String {
        "Highest Priority".to_string()
    }

    fn clone_box(&self) -> Box<dyn Priority> {
        Box::new(self.clone())
    }
}

/// Schedules tasks in order of increasing priority: lower priority tasks are
/// scheduled ahead of higher priority tasks.
///
/// Use this priority if you hate yourself and want to feel busy.
#[derive(Clone, Serialize, Deserialize)]
pub struct LowestPriority {}

#[typetag::serde]
impl Priority for LowestPriority {
    fn select(&self, queue: &[Task]) -> Option<Task> {
        queue.iter().max_by_key(|t| t.priority).cloned()
    }

    fn string(&self) -> String {
        "Lowest Priority".to_string()
    }

    fn clone_box(&self) -> Box<dyn Priority> {
        Box::new(self.clone())
    }
}

impl Clone for Box<dyn Priority> {
    fn clone(&self) -> Self {
        self.clone_box()
    }
}
