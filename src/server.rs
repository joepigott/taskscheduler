use crate::error::{IOError, SerializationError};
use crate::{NaiveTask, Task, TaskQueue, UpdateTask};
use std::sync::atomic::AtomicBool;
use std::sync::{Arc, Mutex};
use warp::Filter;

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
}

impl Server {
    /// Creates a new `Server` with the given sigterm and task queue.
    pub fn with_queue(queue: SharedQueue) -> Self {
        Self {
            tasks: Arc::clone(&queue),
        }
    }

    /// Spawn a new thread and begin listening for requests.
    pub async fn run(&mut self) {
        let tasks = Arc::clone(&self.tasks);

        let filter = warp::any().map(move || tasks.clone());

        let post = warp::post()
            .and(warp::path("v1"))
            .and(warp::path("tasks"))
            .and(warp::path::end())
            .and(Self::post_json())
            .and(filter.clone())
            .and_then(Self::add_task);

        let get = warp::get()
            .and(warp::path("v1"))
            .and(warp::path("tasks"))
            .and(warp::path::end())
            .and(filter.clone())
            .and_then(Self::get_tasks);

        let put = warp::put()
            .and(warp::path("v1"))
            .and(warp::path("tasks"))
            .and(warp::path::end())
            .and(Self::put_json())
            .and(filter.clone())
            .and_then(Self::update_task);

        let delete = warp::delete()
            .and(warp::path("v1"))
            .and(warp::path("tasks"))
            .and(warp::path::end())
            .and(Self::delete_json())
            .and(filter.clone())
            .and_then(Self::delete_task);

        let routes = post.or(get).or(put).or(delete);

        warp::serve(routes).run(([127, 0, 0, 1], 3030)).await;
    }

    /// Extracts a `NaiveTask` from a `POST` request
    fn post_json() -> impl Filter<Extract = (NaiveTask,), Error = warp::Rejection> + Clone {
        warp::body::content_length_limit(1024 * 16).and(warp::body::json())
    }

    /// Extracts an `UpdateTask` from a `PUT` request
    fn put_json() -> impl Filter<Extract = (UpdateTask,), Error = warp::Rejection> + Clone {
        warp::body::content_length_limit(1024 * 16).and(warp::body::json())
    }

    /// Extracts an ID as a `usize` from a `DELETE` request
    fn delete_json() -> impl Filter<Extract = (usize,), Error = warp::Rejection> + Clone {
        warp::body::content_length_limit(1024 * 16).and(warp::body::json())
    }

    /// Adds a task to the queue. A successful operation will reply with a 200 
    /// OK status.
    async fn add_task(
        task: NaiveTask,
        queue: SharedQueue,
    ) -> Result<impl warp::Reply, warp::Rejection> {
        let mut queue = queue.lock().map_err(|_| warp::reject::custom(IOError))?;

        let task = Task::from_naive(task, queue.new_id());
        queue.add(task);

        Ok(warp::reply::with_status(
            "Item successfully added",
            warp::http::StatusCode::CREATED,
        ))
    }

    /// Replies with a serialized representation of the entire contents of the
    /// queue. A successful operation will respond with the requested data 
    /// along with a 200 OK status.
    async fn get_tasks(queue: SharedQueue) -> Result<impl warp::Reply, warp::Rejection> {
        let queue = queue.lock().map_err(|_| warp::reject::custom(IOError))?;

        match bincode::serialize(&queue.clone()) {
            Ok(data) => Ok(warp::reply::with_status(data, warp::http::StatusCode::OK)),
            Err(e) => {
                eprintln!("{e}");
                Err(warp::reject::custom(SerializationError))
            }
        }
    }

    /// Updates a task in the queue. A successful operation will reply with a
    /// 200 OK status.
    async fn update_task(
        updates: UpdateTask,
        queue: SharedQueue,
    ) -> Result<impl warp::Reply, warp::Rejection> {
        let mut queue = queue.lock().map_err(|_| warp::reject::custom(IOError))?;
        let task = match queue.get_mut(updates.id) {
            Some(task) => task,
            None => {
                eprintln!("Task {} does not exist", updates.id);
                return Err(warp::reject::reject());
            }
        };

        // update existing fields

        if let Some(title) = updates.title {
            task.title = title;
        }
        if let Some(deadline) = updates.deadline {
            task.deadline = deadline;
        }
        if let Some(duration) = updates.duration {
            task.duration = duration;
        }
        if let Some(priority) = updates.priority {
            task.priority = priority;
        }

        Ok(warp::reply::with_status(
            "Item successfully updated",
            warp::http::StatusCode::CREATED,
        ))
    }

    /// Deletes a task from the queue. A successful operation will reply with a
    /// 200 OK status.
    async fn delete_task(
        id: usize,
        queue: SharedQueue,
    ) -> Result<impl warp::Reply, warp::Rejection> {
        let mut queue = queue.lock().map_err(|_| warp::reject::custom(IOError))?;

        queue.delete(id)?;

        Ok(warp::reply::with_status(
            "Item successfully deleted",
            warp::http::StatusCode::OK,
        ))
    }
}
