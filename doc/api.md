# Task Scheduler API

This document will go over the general design and usage of the Task Scheduler
API.

## Specification

| Request   | Success | Client Failure | Server Failure |
|-----------|---------|----------------|----------------|
| `POST`    | `201`   | `400`          | `500`          |
| `PUT`     | `201`   | `400`          | `500`          |
| `GET`     | `200`   | `404`          | `503`          |
| `DELETE`  | `200`   | `400`          | `500`          |

### `POST`

A `POST` request should contain a serialized `NaiveTask` instance. This will be
deserialized and processed by the server and will be assigned the lowest unique
ID not in use.

* Success: `201`
* Client failure: `400`
* Server failure: `500`

### `PUT`

A `PUT` request should contain a serialized `UpdateTask` instance with the
appropriate ID and fields to be updated.

* Success: `201`
* Client failure: `400`
* Server failure: `500`

### `GET`

A `GET` request should contain no body. The server will respond with a 
serialized vector of `Task` objects, with which the client can do what they 
please.

* Success: `200`
* Client failure: `404`
* Server failure: `503`

### `DELETE`

A `DELETE` request should contain the ID of the task to be deleted. This action
is not recoverable.

* Success: `200`
* Client failure: `400`
* Server failure: `500`
