use crate::error::{IOError, SerializationError, ServerError, TaskNotFound};
use crate::vars;
use crate::{NaiveTask, SharedQueue, Task, UpdateTask};
use piglog::{error, info};
use std::sync::Arc;
use warp::Filter;

/// `Server` handles all communication with clients. This includes waiting for
/// requests, updating shared resources, and sending responses.
pub struct Server {
    tasks: SharedQueue,
}

impl Server {
    /// Creates a new `Server` with the given task queue.
    pub fn with_queue(queue: SharedQueue) -> Self {
        Self {
            tasks: Arc::clone(&queue),
        }
    }

    /// Spawns a new thread and begin listening for requests. This thread does
    /// *not* exit gracefully as it has no cleanup, so you should exit the
    /// thread forcibly through whatever async runtime you're using.
    pub async fn run(&mut self) -> Result<(), ServerError> {
        info!("Entered server thread");

        let tasks: SharedQueue = Arc::clone(&self.tasks);

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

        let enable = warp::post()
            .and(warp::path("v1"))
            .and(warp::path("tasks"))
            .and(warp::path("enable"))
            .and(warp::path::end())
            .and(filter.clone())
            .and_then(Self::enable);

        let disable = warp::post()
            .and(warp::path("v1"))
            .and(warp::path("tasks"))
            .and(warp::path("disable"))
            .and(warp::path::end())
            .and(filter.clone())
            .and_then(Self::disable);

        let active = warp::get()
            .and(warp::path("v1"))
            .and(warp::path("tasks"))
            .and(warp::path("active"))
            .and(warp::path::end())
            .and(filter.clone())
            .and_then(Self::active);

        let routes = post
            .or(get)
            .or(put)
            .or(delete)
            .or(enable)
            .or(disable)
            .or(active);

        let address = match vars::server_address() {
            Ok(address) => address,
            Err(e) => {
                return Err(ServerError(e));
            }
        };

        warp::serve(routes).run(address).await;

        Ok(())
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
        info!("Adding task {}", task.title);

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
        info!("Fetching tasks");

        let queue = queue.lock().map_err(|_| warp::reject::custom(IOError))?;

        match serde_json::to_vec(&queue.clone()) {
            Ok(data) => Ok(warp::reply::with_status(data, warp::http::StatusCode::OK)),
            Err(e) => {
                error!("{e}");
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
        info!("Updating task {}", updates.id);

        let mut queue = queue.lock().map_err(|_| warp::reject::custom(IOError))?;
        let task = match queue.get_mut(updates.id) {
            Some(task) => task,
            None => {
                error!("Task {} does not exist", updates.id);
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
        info!("Deleting task {id}");

        let mut queue = queue.lock().map_err(|_| warp::reject::custom(IOError))?;
        queue.delete(id)?;

        Ok(warp::reply::with_status(
            "Item successfully deleted",
            warp::http::StatusCode::OK,
        ))
    }

    /// Enables the scheduler, which will start executing scheduling logic.
    async fn enable(queue: SharedQueue) -> Result<impl warp::Reply, warp::Rejection> {
        info!("Enabling scheduler");

        let mut queue = queue.lock().map_err(|_| warp::reject::custom(IOError))?;
        queue.enabled = true;

        Ok(warp::reply::with_status(
            "Scheduler successfully enabled",
            warp::http::StatusCode::OK,
        ))
    }

    /// Disables the scheduler, which will stop executing scheduling logic.
    async fn disable(queue: SharedQueue) -> Result<impl warp::Reply, warp::Rejection> {
        info!("Disabling scheduler");

        let mut queue = queue.lock().map_err(|_| warp::reject::custom(IOError))?;
        queue.enabled = false;

        Ok(warp::reply::with_status(
            "Scheduler successfully disabled",
            warp::http::StatusCode::OK,
        ))
    }

    /// Fetches the active task.
    async fn active(queue: SharedQueue) -> Result<impl warp::Reply, warp::Rejection> {
        info!("Fetching active task");

        let queue = queue.lock().map_err(|_| warp::reject::custom(IOError))?;
        if let Some(task) = queue.select() {
            let data = serde_json::to_vec(&task).map_err(|_| warp::reject::custom(IOError))?;
            Ok(warp::reply::with_status(data, warp::http::StatusCode::OK))
        } else {
            Err(warp::reject::custom(TaskNotFound))
        }
    }
}
