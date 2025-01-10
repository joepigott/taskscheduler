use chrono::{Duration, NaiveDateTime};

type Priority = u8;

/// `Task` contains information about a single task, including its ID, title,
/// deadline, duration, and priority.
pub struct Task {
    id: usize,
    pub title: String,
    pub deadline: NaiveDateTime,
    pub duration: Duration,
    pub priority: Priority,
    pub active: bool,
}

impl Task {
    /// Creates a new `Task` with the provided information.
    pub fn new(
        id: usize,
        title: String,
        deadline: NaiveDateTime,
        duration: Duration,
        priority: Priority,
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
    pub fn from_naive_task(task: NaiveTask, id: usize) -> Self {
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
pub struct NaiveTask {
    pub title: String,
    pub deadline: NaiveDateTime,
    pub duration: Duration,
    pub priority: Priority,
}

impl NaiveTask {
    /// Creates a new `NaiveTask` with the provided information.
    pub fn new(
        title: String,
        deadline: NaiveDateTime,
        duration: Duration,
        priority: Priority,
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
pub struct UpdateTask {
    pub id: u8,
    pub title: Option<String>,
    pub deadline: Option<NaiveDateTime>,
    pub duration: Option<Duration>,
    pub priority: Option<Priority>,
}

impl UpdateTask {
    /// Creates a new `UpdateTask` with the given ID. Use associated builder
    /// methods to add information.
    pub fn new(id: u8) -> Self {
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
    pub fn with_priority(mut self, priority: Priority) -> Self {
        self.priority = Some(priority);
        self
    }
}

/// Possible options to determine the `TaskQueue` priority selection.
pub enum QueuePriority {
    /// Select the task with the closest deadline.
    Deadline,

    /// Select the task with the shortest remaining time.
    ShortestDuration,

    /// Select the task with the longest remaining time.
    LongestDuration,

    /// Select the task with the highest priority.
    Priority,
}

/// A `TaskQueue` is a priority queue whose priority can be changed on the fly.
/// Instead of ordering the tasks based on priority, the priority simply 
/// changes the selection algorithm.
pub struct TaskQueue {
    data: Vec<Task>,
    priority: QueuePriority,
}

impl TaskQueue {
    /// Creates a new `TaskQueue` with the default priority of `Deadline`.
    pub fn new() -> Self {
        Self {
            data: Vec::new(),
            priority: QueuePriority::Deadline
        }
    }

    /// Creates a new `TaskQueue` with the given queue priority.
    pub fn with_priority(priority: QueuePriority) -> Self {
        Self {
            data: Vec::new(),
            priority
        }
    }

    /// Add a new `Task` to the queue.
    pub fn add(&mut self, task: Task) {
        self.data.push(task);
    }

    /// Remove a `Task` from the queue. Order of removal depends on the current
    /// queue priority.
    pub fn remove(&mut self) -> Option<Task> {
        match self.priority {
            QueuePriority::Deadline => self.remove_deadline(),
            QueuePriority::ShortestDuration => self.remove_shortest_duration(),
            QueuePriority::LongestDuration => self.remove_longest_duration(),
            QueuePriority::Priority => self.remove_priority(),
        }
    }

    /// Removes the task with the closest deadline.
    fn remove_deadline(&mut self) -> Option<Task> {
        if let Some((i, task)) = self.data.iter().enumerate().min_by_key(|(_, t)| t.deadline) {
            println!("Removing task {} (\"{}\")", task.id, task.title);
            Some(self.data.remove(i))
        } else {
            None
        }
    }

    /// Removes the task with the shortest remaining duration.
    fn remove_shortest_duration(&mut self) -> Option<Task> {
        if let Some((i, task)) = self.data.iter().enumerate().min_by_key(|(_, t)| t.duration) {
            println!("Removing task {} (\"{}\")", task.id, task.title);
            Some(self.data.remove(i))
        } else {
            None
        }
    }

    /// Removes the task with the longest remaining duration.
    fn remove_longest_duration(&mut self) -> Option<Task> {
        if let Some((i, task)) = self.data.iter().enumerate().max_by_key(|(_, t)| t.duration) {
            println!("Removing task {} (\"{}\")", task.id, task.title);
            Some(self.data.remove(i))
        } else {
            None
        }
    }

    /// Removes the task with the highest priority (lowest value).
    fn remove_priority(&mut self) -> Option<Task> {
        if let Some((i, task)) = self.data.iter().enumerate().min_by_key(|(_, t)| t.priority) {
            println!("Removing task {} (\"{}\")", task.id, task.title);
            Some(self.data.remove(i))
        } else {
            None
        }
    }
}

impl Default for TaskQueue {
    fn default() -> Self {
        Self::new()
    }
}
