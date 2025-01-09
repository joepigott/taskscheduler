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
    /// Creates a new `UpdateTask` with the provided information.
    pub fn new(id: u8, title: Option<String>, deadline: Option<NaiveDateTime>, duration: Option<Duration>, priority: Option<Priority>) -> Self {
        Self {
            id,
            title,
            deadline,
            duration,
            priority
        }
    }
}
