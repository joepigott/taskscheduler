# Task Scheduler API

This document will go over the general design and usage of the Task Scheduler
API.

## Specification

| Request   | Location               | Effect                         | Success | Client Failure | Server Failure |
|-----------|------------------------|--------------------------------|---------|----------------|----------------|
| `POST`    | `/api/tasks/`          | Adds a task to the queue       | `201`   | `400`          | `500`          |
| `PUT`     | `/api/tasks/`          | Updates a task in the queue    | `201`   | `400`          | `500`          |
| `GET`     | `/api/tasks/`          | Fetches the queue contents     | `200`   | `404`          | `503`          |
| `DELETE`  | `/api/tasks/`          | Deletes a task from the queue  | `200`   | `400`          | `500`          |
| `POST`    | `/api/tasks/enable`    | Enables the scheduler          | `200`   | `400`          | `500`          |
| `POST`    | `/api/tasks/disable`   | Disables the scheduler         | `200`   | `400`          | `500`          |
| `GET`     | `/api/tasks/active`    | Fetches the active task        | `200`   | `404`          | `503`          |
| `GET`     | `/api/tasks/status`    | Fetches the scheduler status   | `200`   | `400`          | `500`          |
| `GET`     | `/api/tasks/priority`  | Fetches the scheduler priority | `200`   | `400`          | `500`          |
| `PUT`     | `/api/tasks/priority`  | Sets the scheduler priority    | `200`   | `400`          | `500`          |
| `PUT`     | `/api/tasks/complete`  | Marks a task as complete       | `200`   | `400`          | `500`          |
| `DELETE`  | `/api/tasks/complete`  | Deletes a completed task       | `200`   | `400`          | `500`          |
