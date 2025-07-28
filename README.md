# Task Scheduler

Task Scheduler does what it says: it schedules tasks. This repository contains
the shared resources required by the client and server, as well as some
runtime-agnostic server implementation. I've also written reference 
implementations for the [client](https://github.com/joepigott/tasks) and
[server](https://github.com/joepigott/tasksd).

## Roadmap

The current #1 priority is authentication and support for `https`. I will try
my best to get this implemented as soon as possible, but I am a student so 
there will likely be delays.

## Motivation

Upon finishing a college course in operating systems, I realized that I am
terrible at scheduling tasks in an efficient way. So, I decided to create a
scheduler to algorithmically select the order in which I should work on tasks.
Hopefully this takes away some mental overhead, and allows one to focus on the
actual task rather than stress about getting multiple things done.

## API

*NOTE: there is currently **no authentication or encryption** for communicating
with the server. All requests are made over plain http. Security is currently
the main focus for the next release.*

The API endpoints are as follows:

| Request   | Location               | Effect                         | Success | Client Failure | Server Failure |
|-----------|------------------------|--------------------------------|---------|----------------|----------------|
| `POST`    | `/api/tasks/`          | Adds a task to the queue       | `201`   | `400`          | `500`          |
| `PUT`     | `/api/tasks/`          | Updates a task in the queue    | `201`   | `400`          | `500`          |
| `GET`     | `/api/tasks/`          | Fetches the queue contents     | `200`   | `404`          | `500`          |
| `DELETE`  | `/api/tasks/`          | Deletes a task from the queue  | `200`   | `404`          | `500`          |
| `PUT`     | `/api/tasks/enable`    | Enables the scheduler          | `200`   | `400`          | `500`          |
| `PUT`     | `/api/tasks/disable`   | Disables the scheduler         | `200`   | `400`          | `500`          |
| `GET`     | `/api/tasks/active`    | Fetches the active task        | `200`   | `404`          | `500`          |
| `GET`     | `/api/tasks/status`    | Fetches the scheduler status   | `200`   | `400`          | `500`          |
| `GET`     | `/api/tasks/priority`  | Fetches the scheduler priority | `200`   | `400`          | `500`          |
| `PUT`     | `/api/tasks/priority`  | Sets the scheduler priority    | `201`   | `400`          | `500`          |
| `PUT`     | `/api/tasks/complete`  | Marks a task as complete       | `200`   | `404`          | `500`          |
| `DELETE`  | `/api/tasks/complete`  | Deletes a completed task       | `200`   | `404`          | `500`          |

## Scheduling Algorithms

If I'm being honest, I'm not exactly sure of the most efficient way to schedule
tasks in a real-world, real-time scenario. So, the task scheduler is written in
such a way that the actual scheduling algorithm used by the priority queue is
hot-swappable; these are defined by the `Priority` trait. For example, 
implementing a "shortest job first" priority algorithm amounts to implementing
the following methods:
```rust
#[derive(Clone, Serialize, Deserialize)]
pub struct SJF {}

#[typetag::serde]
impl Priority for SJF {
    fn select(&self, queue: &[Task]) -> Option<Task> {
        queue.iter().min_by_key(|t| t.duration).clone()
    }

    fn string(&self) -> String {
        "Shortest Job First".to_string()
    }

    fn clone_box(&self) -> Box<dyn Priority> {
        Box::new(self.clone())
    }
}
```
The `select()` method defines the algorithm for selecting the task to be
executed.

`clone_box()` is an unfortunate consequence of serializing/deserializing trait
objects. The above implementation will do just fine. If you have a better
solution for this, please open a pull request.
