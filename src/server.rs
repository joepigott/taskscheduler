use crate::{NaiveTask, Task, TaskQueue};
use std::sync::atomic::AtomicBool;
use std::sync::{Arc, Mutex};
use std::future::Future;
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
    pub async fn run(&mut self) -> impl Future<Output = ()> {
        let tasks = Arc::clone(&self.tasks);

        let filter = warp::any().map(move || tasks.clone());

        let post = warp::post()
            .and(warp::path("v1"))
            .and(warp::path("tasks"))
            .and(warp::path::end())
            .and(Self::post_json())
            .and(filter.clone())
            .and_then(Self::add_task);

        let routes = post;

        warp::serve(routes).run(([127, 0, 0, 1], 3030))
    }

    fn post_json() -> impl Filter<Extract = (NaiveTask,), Error = warp::Rejection> + Clone {
        warp::body::content_length_limit(1024 * 16).and(warp::body::json())
    }

    async fn add_task(
        task: NaiveTask,
        queue: SharedQueue,
    ) -> Result<impl warp::Reply, warp::Rejection> {
        let mut queue = queue.lock().unwrap();
        let task = Task::from_naive_task(task, queue.new_id());
        queue.add(task);

        Ok(warp::reply::with_status(
            "Item successfully added",
            warp::http::StatusCode::CREATED,
        ))
    }
}
