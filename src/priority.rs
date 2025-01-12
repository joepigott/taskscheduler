use crate::Task;
use serde::{Deserialize, Serialize};

#[typetag::serde(tag = "type")]
pub trait Priority: Send + Sync {
    fn pop(&self, queue: &mut Vec<Task>) -> Option<Task>;
    fn peek(&self, queue: &[Task]) -> Option<Task>;
    fn clone_box(&self) -> Box<dyn Priority>;
}

/// Schedules tasks in the order they were added to the queue.
#[derive(Clone, Serialize, Deserialize)]
pub struct FIFO;

#[typetag::serde]
impl Priority for FIFO {
    fn pop(&self, queue: &mut Vec<Task>) -> Option<Task> {
        if !queue.is_empty() {
            Some(queue.remove(0))
        } else {
            None
        }
    }

    fn peek(&self, queue: &[Task]) -> Option<Task> {
        queue.first().cloned()
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
    fn pop(&self, queue: &mut Vec<Task>) -> Option<Task> {
        if let Some((i, _)) = queue.iter().enumerate().min_by_key(|(_, t)| t.deadline) {
            Some(queue.remove(i))
        } else {
            None
        }
    }

    fn peek(&self, queue: &[Task]) -> Option<Task> {
        queue.iter().min_by_key(|t| t.deadline).cloned()
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
    fn pop(&self, queue: &mut Vec<Task>) -> Option<Task> {
        if let Some((i, _)) = queue.iter().enumerate().min_by_key(|(_, t)| t.duration) {
            Some(queue.remove(i))
        } else {
            None
        }
    }

    fn peek(&self, queue: &[Task]) -> Option<Task> {
        queue.iter().min_by_key(|t| t.duration).cloned()
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
    fn pop(&self, queue: &mut Vec<Task>) -> Option<Task> {
        if let Some((i, _)) = queue.iter().enumerate().max_by_key(|(_, t)| t.duration) {
            Some(queue.remove(i))
        } else {
            None
        }
    }

    fn peek(&self, queue: &[Task]) -> Option<Task> {
        queue.iter().max_by_key(|t| t.duration).cloned()
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
    fn pop(&self, queue: &mut Vec<Task>) -> Option<Task> {
        if let Some((i, _)) = queue.iter().enumerate().min_by_key(|(_, t)| t.priority) {
            Some(queue.remove(i))
        } else {
            None
        }
    }

    fn peek(&self, queue: &[Task]) -> Option<Task> {
        queue.iter().min_by_key(|t| t.priority).cloned()
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
    fn pop(&self, queue: &mut Vec<Task>) -> Option<Task> {
        if let Some((i, _)) = queue.iter().enumerate().max_by_key(|(_, t)| t.priority) {
            Some(queue.remove(i))
        } else {
            None
        }
    }

    fn peek(&self, queue: &[Task]) -> Option<Task> {
        queue.iter().max_by_key(|t| t.priority).cloned()
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
