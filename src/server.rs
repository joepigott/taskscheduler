use crate::error::{IOError, SerializationError, ServerError, TaskNotFound};
use crate::priority::Priority;
use crate::vars;
use crate::{NaiveTask, SharedQueue, Task, UpdateTask};
use piglog::{error, info};
use std::convert::Infallible;
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
        info!("Starting server...");

        let tasks: SharedQueue = Arc::clone(&self.tasks);

        let filter = warp::any().map(move || tasks.clone());

        let post = warp::post()
            .and(warp::path("api"))
            .and(warp::path("tasks"))
            .and(warp::path::end())
            .and(Self::post_json())
            .and(filter.clone())
            .and_then(Self::add_task);

        let get = warp::get()
            .and(warp::path("api"))
            .and(warp::path("tasks"))
            .and(warp::path::end())
            .and(filter.clone())
            .and_then(Self::get_tasks);

        let put = warp::put()
            .and(warp::path("api"))
            .and(warp::path("tasks"))
            .and(warp::path::end())
            .and(Self::put_json())
            .and(filter.clone())
            .and_then(Self::update_task);

        let delete = warp::delete()
            .and(warp::path("api"))
            .and(warp::path("tasks"))
            .and(warp::path::end())
            .and(Self::id_json())
            .and(filter.clone())
            .and_then(Self::delete_task);

        let enable = warp::post()
            .and(warp::path("api"))
            .and(warp::path("tasks"))
            .and(warp::path("enable"))
            .and(warp::path::end())
            .and(filter.clone())
            .and_then(Self::enable);

        let disable = warp::post()
            .and(warp::path("api"))
            .and(warp::path("tasks"))
            .and(warp::path("disable"))
            .and(warp::path::end())
            .and(filter.clone())
            .and_then(Self::disable);

        let active = warp::get()
            .and(warp::path("api"))
            .and(warp::path("tasks"))
            .and(warp::path("active"))
            .and(warp::path::end())
            .and(filter.clone())
            .and_then(Self::active);

        let status = warp::get()
            .and(warp::path("api"))
            .and(warp::path("tasks"))
            .and(warp::path("status"))
            .and(warp::path::end())
            .and(filter.clone())
            .and_then(Self::status);

        let set_priority = warp::put()
            .and(warp::path("api"))
            .and(warp::path("tasks"))
            .and(warp::path("priority"))
            .and(warp::path::end())
            .and(Self::priority_json())
            .and(filter.clone())
            .and_then(Self::set_priority);

        let get_priority = warp::get()
            .and(warp::path("api"))
            .and(warp::path("tasks"))
            .and(warp::path("priority"))
            .and(warp::path::end())
            .and(filter.clone())
            .and_then(Self::get_priority);

        let complete = warp::put()
            .and(warp::path("api"))
            .and(warp::path("tasks"))
            .and(warp::path("complete"))
            .and(warp::path::end())
            .and(Self::id_json())
            .and(filter.clone())
            .and_then(Self::complete);

        let del_complete = warp::delete()
            .and(warp::path("api"))
            .and(warp::path("tasks"))
            .and(warp::path("complete"))
            .and(warp::path::end())
            .and(Self::id_json())
            .and(filter.clone())
            .and_then(Self::del_complete);

        let routes = post
            .or(get)
            .or(put)
            .or(delete)
            .or(enable)
            .or(disable)
            .or(active)
            .or(status)
            .or(set_priority)
            .or(get_priority)
            .or(complete)
            .or(del_complete)
            .recover(Self::handle_rejection);

        let address = vars::server_address().map_err(ServerError)?;
        if !vars::is_available(address) {
            return Err(ServerError("Address is already in use".to_string()));
        }

        info!("Server listening on {address}");
        warp::serve(routes).run(address).await;

        Ok(())
    }

    /// Extracts a `NaiveTask` from a `POST` request.
    fn post_json() -> impl Filter<Extract = (NaiveTask,), Error = warp::Rejection> + Clone {
        warp::body::content_length_limit(1024 * 16).and(warp::body::json())
    }

    /// Extracts an `UpdateTask` from a `PUT` request.
    fn put_json() -> impl Filter<Extract = (UpdateTask,), Error = warp::Rejection> + Clone {
        warp::body::content_length_limit(1024 * 16).and(warp::body::json())
    }

    /// Extracts an ID as a `usize` from a `DELETE` request.
    fn id_json() -> impl Filter<Extract = (usize,), Error = warp::Rejection> + Clone {
        warp::body::content_length_limit(1024 * 16).and(warp::body::json())
    }

    /// Extracts a `Box<dyn Priority>` from a `PUT` request.
    fn priority_json(
    ) -> impl Filter<Extract = (Box<dyn Priority>,), Error = warp::Rejection> + Clone {
        warp::body::content_length_limit(1024 * 16).and(warp::body::json())
    }

    /// Adds a task to the queue.
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
    /// queue.
    async fn get_tasks(queue: SharedQueue) -> Result<impl warp::Reply, warp::Rejection> {
        info!("Fetching tasks");

        let queue = queue.lock().map_err(|_| warp::reject::custom(IOError))?;

        if !queue.is_empty() {
            match serde_json::to_vec(&queue.clone()) {
                Ok(data) => Ok(warp::reply::with_status(data, warp::http::StatusCode::OK)),
                Err(e) => {
                    error!("{e}");
                    Err(warp::reject::custom(SerializationError))
                }
            }
        } else {
            Err(warp::reject::custom(TaskNotFound))
        }
    }

    /// Updates a task in the queue.
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

    /// Deletes a task from the queue.
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

    /// Fetches the scheduler status (enabled/disabled).
    async fn status(queue: SharedQueue) -> Result<impl warp::Reply, warp::Rejection> {
        info!("Fetching scheduler status");

        let queue = queue.lock().map_err(|_| warp::reject::custom(IOError))?;
        let data = serde_json::to_vec(&queue.enabled).map_err(|_| warp::reject::custom(IOError))?;
        Ok(warp::reply::with_status(data, warp::http::StatusCode::OK))
    }

    /// Applies the provided priority to the task queue.
    async fn set_priority(
        priority: Box<dyn Priority>,
        queue: SharedQueue,
    ) -> Result<impl warp::Reply, warp::Rejection> {
        info!("Updating task queue priority");

        let mut queue = queue.lock().map_err(|_| warp::reject::custom(IOError))?;
        queue.priority = priority;

        Ok(warp::reply::with_status(
            "Task queue priority successfully updated",
            warp::http::StatusCode::CREATED,
        ))
    }

    /// Fetches the current scheduler priority
    async fn get_priority(queue: SharedQueue) -> Result<impl warp::Reply, warp::Rejection> {
        info!("Fetching scheduler priority");

        let queue = queue.lock().map_err(|_| warp::reject::custom(IOError))?;
        let reply = serde_json::to_string(&queue.priority)
            .map_err(|_| warp::reject::custom(SerializationError))?;

        Ok(warp::reply::with_status(reply, warp::http::StatusCode::OK))
    }

    /// Marks the task with the given ID as complete.
    async fn complete(id: usize, queue: SharedQueue) -> Result<impl warp::Reply, warp::Rejection> {
        info!("Marking task {id} as complete");

        let mut queue = queue.lock().map_err(|_| warp::reject::custom(IOError))?;
        let task = queue
            .get_mut(id)
            .ok_or(warp::reject::custom(TaskNotFound))?
            .to_owned();
        queue
            .delete(task.id)
            .map_err(|_| warp::reject::custom(TaskNotFound))?;
        let c_task = Task::new(
            queue.new_id_completed(),
            task.title,
            task.deadline,
            task.duration,
            task.priority,
        );
        queue.add_completed(c_task);

        Ok(warp::reply::with_status(
            "Task marked as completed",
            warp::http::StatusCode::OK,
        ))
    }

    /// Deletes a task from the completed list.
    async fn del_complete(
        id: usize,
        queue: SharedQueue,
    ) -> Result<impl warp::Reply, warp::Rejection> {
        info!("Deleting task {id}");

        let mut queue = queue.lock().map_err(|_| warp::reject::custom(IOError))?;
        queue.delete_completed(id)?;

        Ok(warp::reply::with_status(
            "Item successfully deleted",
            warp::http::StatusCode::OK,
        ))
    }

    /// Transforms rejections into proper server replies.
    async fn handle_rejection(err: warp::Rejection) -> Result<impl warp::Reply, Infallible> {
        let message;
        let code;

        if err.find::<IOError>().is_some() {
            message = "An IO error occurred on the server";
            code = warp::http::StatusCode::INTERNAL_SERVER_ERROR;
        } else if err.find::<SerializationError>().is_some() {
            message = "A serialization error occurred on the server";
            code = warp::http::StatusCode::INTERNAL_SERVER_ERROR;
        } else if err.find::<TaskNotFound>().is_some() {
            message = "The specified task doesn't exist";
            code = warp::http::StatusCode::NOT_FOUND;
        } else {
            message = "An unknown error occurred. Sorry!";
            code = warp::http::StatusCode::INTERNAL_SERVER_ERROR;
        }

        Ok(warp::reply::with_status(message, code))
    }
}
