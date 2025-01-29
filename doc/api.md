# Task Scheduler API

This document will go over the general design and usage of the Task Scheduler
API.

## Specification

| Request   | Location              | Effect                         | Success | Client Failure | Server Failure |
|-----------|-----------------------|--------------------------------|---------|----------------|----------------|
| `POST`    | `/v1/tasks/`          | Adds a task to the queue       | `201`   | `400`          | `500`          |
| `PUT`     | `/v1/tasks/`          | Updates a task in the queue    | `201`   | `400`          | `500`          |
| `GET`     | `/v1/tasks/`          | Fetches the queue contents     | `200`   | `404`          | `503`          |
| `DELETE`  | `/v1/tasks/`          | Deletes a task from the queue  | `200`   | `400`          | `500`          |
| `POST`    | `/v1/tasks/enable`    | Enables the scheduler          | `200`   | `400`          | `500`          |
| `POST`    | `/v1/tasks/disable`   | Disables the scheduler         | `200`   | `400`          | `500`          |
| `GET`     | `/v1/tasks/active`    | Fetches the active task        | `200`   | `404`          | `503`          |
| `GET`     | `/v1/tasks/status`    | Fetches the scheduler status   | `200`   | `400`          | `500`          |
| `PUT`    | `/v1/tasks/priority`   | Sets the scheduler priority    | `200`   | `400`          | `500`          |
| `PUT`    | `/v1/tasks/complete `  | Marks a task as complete       | `200`   | `400`          | `500`          |
