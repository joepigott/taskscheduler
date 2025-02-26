use chrono::{Duration, NaiveDateTime};
use priority::{Deadline, Priority};
use serde::{Deserialize, Serialize};
use std::fmt::Display;
use std::str::FromStr;
use std::sync::{Arc, Mutex};

pub mod error;
pub mod priority;
pub mod scheduler;
pub mod server;
pub mod vars;

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
pub enum PriorityLevel {
    Urgent,
    High,
    Normal,
    Low,
}

impl Display for PriorityLevel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                PriorityLevel::Urgent => "Urgent",
                PriorityLevel::High => "High",
                PriorityLevel::Normal => "Normal",
                PriorityLevel::Low => "Low",
            }
        )
    }
}

impl FromStr for PriorityLevel {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "urgent" => Ok(PriorityLevel::Urgent),
            "high" => Ok(PriorityLevel::High),
            "normal" => Ok(PriorityLevel::Normal),
            "low" => Ok(PriorityLevel::Low),
            _ => Err("Unknown priority level".to_string()),
        }
    }
}

/// `Task` contains information about a single task, including its ID, title,
/// deadline, duration, and priority.
#[derive(Clone, Serialize, Deserialize)]
pub struct Task {
    id: usize,
    pub title: String,
    pub deadline: NaiveDateTime,
    pub duration: Duration,
    pub priority: PriorityLevel,
    pub active: bool,
    pub completed: bool,
}

impl Task {
    /// Creates a new `Task` with the provided information.
    pub fn new(
        id: usize,
        title: String,
        deadline: NaiveDateTime,
        duration: Duration,
        priority: PriorityLevel,
    ) -> Self {
        Self {
            id,
            title,
            deadline,
            duration,
            priority,
            active: false,
            completed: false,
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
            completed: false,
        }
    }

    /// Returns this `Task`'s ID
    pub fn id(&self) -> usize {
        self.id
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
    pub priority: PriorityLevel,
}

impl NaiveTask {
    /// Creates a new `NaiveTask` with the provided information.
    pub fn new(
        title: String,
        deadline: NaiveDateTime,
        duration: Duration,
        priority: PriorityLevel,
    ) -> Self {
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
    pub priority: Option<PriorityLevel>,
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
    pub fn with_title(mut self, title: Option<String>) -> Self {
        self.title = title;
        self
    }

    /// Adds a deadline to the `UpdateTask` and returns it.
    pub fn with_deadline(mut self, deadline: Option<NaiveDateTime>) -> Self {
        self.deadline = deadline;
        self
    }

    /// Adds a duration to the `UpdateTask` and returns it.
    pub fn with_duration(mut self, duration: Option<Duration>) -> Self {
        self.duration = duration;
        self
    }

    /// Adds a priority to the `UpdateTask` and returns it.
    pub fn with_priority(mut self, priority: Option<PriorityLevel>) -> Self {
        self.priority = priority;
        self
    }
}

/// A `TaskQueue` is a priority queue whose priority can be changed on the fly.
/// Instead of ordering the tasks based on priority, the priority simply
/// changes the selection algorithm.
#[derive(Clone, Serialize, Deserialize)]
pub struct TaskQueue {
    tasks: Vec<Task>,
    completed: Vec<Task>,
    priority: Box<dyn Priority>,
    pub enabled: bool,
}

impl TaskQueue {
    /// Creates a new `TaskQueue` with the default priority of `Deadline`.
    pub fn new() -> Self {
        Self {
            tasks: Vec::new(),
            completed: Vec::new(),
            priority: Box::new(Deadline {}),
            enabled: false,
        }
    }

    /// Creates a new `TaskQueue` with the given queue priority.
    pub fn with_priority<P: Priority + 'static>(priority: P) -> Self {
        Self {
            tasks: Vec::new(),
            completed: Vec::new(),
            priority: Box::new(priority),
            enabled: false,
        }
    }

    /// Returns a string representing the current queue priority
    pub fn show_priority(&self) -> String {
        self.priority.string()
    }

    /// Finds and returns the lowest unused ID.
    pub fn new_id(&self) -> usize {
        use std::collections::HashSet;

        let ids: HashSet<usize> = self.tasks.iter().map(|t| t.id).collect();
        (1..).find(|id| !ids.contains(id)).unwrap()
    }

    /// Finds and returns the lowest unused ID in the completed list.
    pub fn new_id_completed(&self) -> usize {
        use std::collections::HashSet;

        let ids: HashSet<usize> = self.completed.iter().map(|t| t.id).collect();
        (1..).find(|id| !ids.contains(id)).unwrap()
    }

    /// Returns an iterator over the contents of the queue.
    pub fn iter(&self) -> TaskQueueIterator {
        TaskQueueIterator {
            task_queue: self,
            index: 0,
        }
    }

    /// Returns an iterator over the contents of the completed tasks.
    pub fn iter_completed(&self) -> TaskQueueIteratorCompleted {
        TaskQueueIteratorCompleted {
            task_queue: self,
            index: 0,
        }
    }

    /// Add a new `Task` to the queue.
    pub fn add(&mut self, task: Task) {
        self.tasks.push(task);
    }

    /// Add a `Task` to the completed list.
    pub fn add_completed(&mut self, task: Task) {
        self.completed.push(task);
    }

    /// Returns the next task based on the current priority algorithm.
    pub fn select(&self) -> Option<Task> {
        self.priority.select(&self.tasks)
    }

    /// Remove the `i`th task from the queue.
    pub fn remove(&mut self, i: usize) -> Option<Task> {
        if i < self.tasks.len() {
            Some(self.tasks.remove(i))
        } else {
            None
        }
    }

    /// Remove the `i`th task from the completed list.
    pub fn remove_completed(&mut self, i: usize) -> Option<Task> {
        if i < self.completed.len() {
            Some(self.completed.remove(i))
        } else {
            None
        }
    }

    /// Returns a reference to the `i`th task *without* removing it from the
    /// queue.
    pub fn nth(&self, i: usize) -> Option<&Task> {
        self.tasks.get(i)
    }

    /// Returns a reference to the `i`th task *without* removing it from the
    /// completed list.
    pub fn nth_completed(&self, i: usize) -> Option<&Task> {
        self.completed.get(i)
    }

    /// Returns a mutable reference to the task corresponding to the given ID.
    pub fn get_mut(&mut self, id: usize) -> Option<&mut Task> {
        self.tasks.iter_mut().find(|t| t.id == id)
    }

    /// Returns a mutable reference to the completed task corresponding to the
    /// given ID.
    pub fn get_mut_completed(&mut self, id: usize) -> Option<&mut Task> {
        self.completed.iter_mut().find(|t| t.id == id)
    }

    /// Deletes the task corresponding to the given ID from the queue. If the
    /// task does not exist, a `TaskNotFound` error is returned.
    pub fn delete(&mut self, id: usize) -> Result<(), error::TaskNotFound> {
        if let Some((i, _)) = self.tasks.iter().enumerate().find(|(_, t)| t.id == id) {
            self.tasks.remove(i);
            Ok(())
        } else {
            Err(error::TaskNotFound)
        }
    }

    /// Deletes the task corresponding to the given ID from the completed list.
    /// If the task does not exist, a `TaskNotFound` error is returned.
    pub fn delete_completed(&mut self, id: usize) -> Result<(), error::TaskNotFound> {
        if let Some((i, _)) = self.completed.iter().enumerate().find(|(_, t)| t.id == id) {
            self.completed.remove(i);
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
        if self.index < self.task_queue.tasks.len() {
            let result = &self.task_queue.tasks[self.index];
            self.index += 1;

            Some(result)
        } else {
            None
        }
    }
}

/// Implements `Iterator` for easy iteration over the completed tasks in a
/// `TaskQueue`.
pub struct TaskQueueIteratorCompleted<'a> {
    task_queue: &'a TaskQueue,
    index: usize,
}

impl<'a> Iterator for TaskQueueIteratorCompleted<'a> {
    type Item = &'a Task;

    fn next(&mut self) -> Option<Self::Item> {
        if self.index < self.task_queue.completed.len() {
            let result = &self.task_queue.completed[self.index];
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
                PriorityLevel::Normal,
            );
            queue.add(task);
        }

        {
            let queue1 = queue.clone();
            assert_eq!(queue1.new_id(), 11);
        }

        {
            let mut queue1 = queue.clone();
            queue1.tasks.remove(2);
            assert_eq!(queue1.new_id(), 3);
        }

        {
            let mut queue1 = queue.clone();
            queue1.tasks.remove(3);
            queue1.tasks.remove(6);
            assert_eq!(queue1.new_id(), 4);
        }
    }
}

pub type SharedQueue = Arc<Mutex<TaskQueue>>;
