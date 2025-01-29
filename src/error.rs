use std::error::Error;
use std::fmt::{Debug, Display};
use std::sync::PoisonError;

/// An error that can occur during serialization or deserialization.
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

/// An error that can occur due to IO operations such as accessing data through
/// a Mutex.
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

/// An error that occurs when an operation requested access to a task but the
/// task does not exist.
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

impl Error for TaskNotFound {}
impl warp::reject::Reject for TaskNotFound {}

/// An error that occurs in the scheduling logic.
pub struct SchedulingError(pub String);

impl Display for SchedulingError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl Debug for SchedulingError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl<T> From<PoisonError<T>> for SchedulingError {
    fn from(value: PoisonError<T>) -> Self {
        Self(value.to_string())
    }
}

impl From<TaskNotFound> for SchedulingError {
    fn from(_: TaskNotFound) -> Self {
        Self("The requested task does not exist".to_string())
    }
}

impl From<String> for SchedulingError {
    fn from(value: String) -> Self {
        Self(value)
    }
}

impl Error for SchedulingError {}

/// An error that occurs in the server logic.
pub struct ServerError(pub String);

impl Display for ServerError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl Debug for ServerError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl Error for ServerError {}
