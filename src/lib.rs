use chrono::{Duration, NaiveDateTime};
use priority::{Deadline, Priority};
use serde::{Deserialize, Serialize};

pub mod error;
pub mod priority;
pub mod server;

/// `Task` contains information about a single task, including its ID, title,
/// deadline, duration, and priority.
#[derive(Clone, Serialize, Deserialize)]
pub struct Task {
    id: usize,
    pub title: String,
    pub deadline: NaiveDateTime,
    pub duration: Duration,
    pub priority: u8,
    pub active: bool,
}

impl Task {
    /// Creates a new `Task` with the provided information.
    pub fn new(
        id: usize,
        title: String,
        deadline: NaiveDateTime,
        duration: Duration,
        priority: u8,
    ) -> Self {
        Self {
            id,
            title,
            deadline,
            duration,
            priority,
            active: false,
        }
    }

    /// Creates a new `Task` from an existing `NaiveTask` and an ID.
    pub fn from_naive(task: NaiveTask, id: usize) -> Self {
        Self {
            id,
            title: task.title,
            deadline: task.deadline,
            duration: task.duration,
            priority: task.priority,
            active: false,
        }
    }
}

impl std::fmt::Display for Task {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}{} - {}\n\tDeadline: {}\n\tTime Remaining: {} hours\n\tPriority: {}\n",
            if self.active { " * " } else { "   " },
            self.id,
            self.title,
            self.deadline,
            self.duration.num_hours(),
            self.priority,
        )
    }
}

/// A `NaiveTask` contains the same information as a `Task`, but lacks an ID
/// and an active flag. This is useful when the client sends task information
/// and the server is responsible for assigning it an ID depending on the tasks
/// already in the queue.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NaiveTask {
    pub title: String,
    pub deadline: NaiveDateTime,
    pub duration: Duration,
    pub priority: u8,
}

impl NaiveTask {
    /// Creates a new `NaiveTask` with the provided information.
    pub fn new(title: String, deadline: NaiveDateTime, duration: Duration, priority: u8) -> Self {
        Self {
            title,
            deadline,
            duration,
            priority,
        }
    }
}

/// An `UpdateTask` requires an ID, and will be sent to the server to update
/// any specified fields associated with that ID.
#[derive(Clone, Serialize, Deserialize)]
pub struct UpdateTask {
    pub id: usize,
    pub title: Option<String>,
    pub deadline: Option<NaiveDateTime>,
    pub duration: Option<Duration>,
    pub priority: Option<u8>,
}

impl UpdateTask {
    /// Creates a new `UpdateTask` with the given ID. Use associated builder
    /// methods to add information.
    pub fn new(id: usize) -> Self {
        Self {
            id,
            title: None,
            deadline: None,
            duration: None,
            priority: None,
        }
    }

    /// Adds a title to the `UpdateTask` and returns it.
    pub fn with_title(mut self, title: String) -> Self {
        self.title = Some(title);
        self
    }

    /// Adds a deadline to the `UpdateTask` and returns it.
    pub fn with_deadline(mut self, deadline: NaiveDateTime) -> Self {
        self.deadline = Some(deadline);
        self
    }

    /// Adds a duration to the `UpdateTask` and returns it.
    pub fn with_duration(mut self, duration: Duration) -> Self {
        self.duration = Some(duration);
        self
    }

    /// Adds a priority to the `UpdateTask` and returns it.
    pub fn with_priority(mut self, priority: u8) -> Self {
        self.priority = Some(priority);
        self
    }
}

/// A `TaskQueue` is a priority queue whose priority can be changed on the fly.
/// Instead of ordering the tasks based on priority, the priority simply
/// changes the selection algorithm.
#[derive(Clone, Serialize, Deserialize)]
pub struct TaskQueue {
    data: Vec<Task>,
    priority: Box<dyn Priority>,
    pub enabled: bool,
}

impl TaskQueue {
    /// Creates a new `TaskQueue` with the default priority of `Deadline`.
    pub fn new() -> Self {
        Self {
            data: Vec::new(),
            priority: Box::new(Deadline {}),
            enabled: false,
        }
    }

    /// Creates a new `TaskQueue` with the given queue priority.
    pub fn with_priority<P: Priority + 'static>(priority: P) -> Self {
        Self {
            data: Vec::new(),
            priority: Box::new(priority),
            enabled: false,
        }
    }

    /// Finds and returns the lowest unused ID.
    pub fn new_id(&self) -> usize {
        use std::collections::HashSet;

        let ids: HashSet<usize> = self.data.iter().map(|t| t.id).collect();
        (1..).find(|id| !ids.contains(id)).unwrap()
    }

    /// Returns an iterator over the contents of the queue.
    pub fn iter(&self) -> TaskQueueIterator {
        TaskQueueIterator {
            task_queue: self,
            index: 0,
        }
    }

    /// Add a new `Task` to the queue.
    pub fn add(&mut self, task: Task) {
        self.data.push(task);
    }

    /// Pop a `Task` from the queue. Order of removal depends on the current
    /// queue priority.
    pub fn pop(&mut self) -> Option<Task> {
        self.priority.pop(&mut self.data)
    }

    /// Returns the `Task` that would be popped from the queue *without*
    /// removing it. The `Task` is cloned, not a reference.
    pub fn peek(&self) -> Option<Task> {
        self.priority.peek(&self.data)
    }

    /// Remove the `i`th task from the queue.
    pub fn remove(&mut self, i: usize) -> Option<Task> {
        if i < self.data.len() {
            Some(self.data.remove(i))
        } else {
            None
        }
    }

    /// Returns a reference to the `i`th task *without* removing it from the
    /// queue.
    pub fn nth(&self, i: usize) -> Option<&Task> {
        self.data.get(i)
    }

    /// Returns a mutable reference to the task corresponding to the given ID.
    pub fn get_mut(&mut self, id: usize) -> Option<&mut Task> {
        self.data.iter_mut().find(|t| t.id == id)
    }

    /// Deletes the task corresponding to the given ID from the queue. If the
    /// task does not exist, a `TaskNotFound` error is returned.
    pub fn delete(&mut self, id: usize) -> Result<(), error::TaskNotFound> {
        if let Some((i, _)) = self.data.iter().enumerate().find(|(_, t)| t.id == id) {
            self.data.remove(i);
            Ok(())
        } else {
            Err(error::TaskNotFound)
        }
    }
}

impl Default for TaskQueue {
    fn default() -> Self {
        Self::new()
    }
}

/// Implements `Iterator` for easy iteration over a `TaskQueue`.
pub struct TaskQueueIterator<'a> {
    task_queue: &'a TaskQueue,
    index: usize,
}

impl<'a> Iterator for TaskQueueIterator<'a> {
    type Item = &'a Task;

    fn next(&mut self) -> Option<Self::Item> {
        if self.index < self.task_queue.data.len() {
            let result = &self.task_queue.data[self.index];
            self.index += 1;

            Some(result)
        } else {
            None
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_new_id() {
        let mut queue = TaskQueue::new();
        for i in 1..=10 {
            let task = Task::new(
                i,
                format!("Task {i}"),
                NaiveDateTime::parse_from_str("01/10/2025 01:00 am", "%m/%d/%Y %M:%H %P").unwrap(),
                Duration::zero(),
                1,
            );
            queue.add(task);
        }

        {
            let queue1 = queue.clone();
            assert_eq!(queue1.new_id(), 11);
        }

        {
            let mut queue1 = queue.clone();
            queue1.data.remove(2);
            assert_eq!(queue1.new_id(), 3);
        }

        {
            let mut queue1 = queue.clone();
            queue1.data.remove(3);
            queue1.data.remove(6);
            assert_eq!(queue1.new_id(), 4);
        }
    }
}
