use std::fmt::{Debug, Display};
use std::sync::PoisonError;

pub struct SerializationError;

impl Display for SerializationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Unable to serialize data")
    }
}

impl Debug for SerializationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Unable to serialize data")
    }
}

impl warp::reject::Reject for SerializationError {}
impl std::error::Error for SerializationError {}

pub struct IOError;

impl Display for IOError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Error retrieving lock for data")
    }
}

impl Debug for IOError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Error retrieving lock for data")
    }
}

impl warp::reject::Reject for IOError {}
impl std::error::Error for IOError {}

pub struct TaskNotFound;

impl Display for TaskNotFound {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "The requested task does not exist")
    }
}

impl Debug for TaskNotFound {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "The requested task does not exist")
    }
}

impl std::error::Error for TaskNotFound {}
impl warp::reject::Reject for TaskNotFound {}

pub enum SchedulingErrorType {
    IOError,
    SyncError,
    LogicError,
}

impl Display for SchedulingErrorType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let result = match self {
            SchedulingErrorType::IOError => "IO Error",
            SchedulingErrorType::SyncError => "Sync Error",
            SchedulingErrorType::LogicError => "Logic Error",
        };
        write!(f, "{result}")
    }
}

pub struct SchedulingError {
    message: String, 
    etype: SchedulingErrorType,
}

impl SchedulingError {
    pub fn new(message: String, etype: SchedulingErrorType) -> Self {
        Self { message, etype }
    }
}

impl Display for SchedulingError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}: {}", self.etype, self.message)
    }
}

impl<T> From<PoisonError<T>> for SchedulingError {
    fn from(value: PoisonError<T>) -> Self {
        Self {
            message: value.to_string(),
            etype: SchedulingErrorType::SyncError,
        }
    }
}

impl From<TaskNotFound> for SchedulingError {
    fn from(_: TaskNotFound) -> Self {
        Self {
            message: "The requested task does not exist".to_string(),
            etype: SchedulingErrorType::LogicError,
        }
    }
}
