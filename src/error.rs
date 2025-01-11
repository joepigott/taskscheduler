use std::fmt::{Debug, Display};

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