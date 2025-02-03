use crate::Task;
use serde::{Deserialize, Serialize};

/// A struct implementing the `Priority` trait can be assigned to a `TaskQueue`
/// to define the method for selecting tasks. The important method is
/// `select()` which defines the actual method of selection. To work properly,
/// the `select()` method should **ignore tasks that are flagged as complete**.
///
/// ## Example: `FIFO`
///
/// ```rust
/// pub struct FIFO {}
///
/// impl Priority for FIFO {
///     fn select(&self, queue: &[Task]) -> Option<Task> {
///         queue.first().cloned()
///     }
/// }
/// ```
///
/// ## `clone_box()`
///
/// The `clone_box()` method is required to satisfy the trait bounds for
/// trait object serialization and deserialization. The following
/// implementation will work just fine:
/// ```rust
/// fn clone_box(&self) -> Box<dyn Priority> {
///     Box::new(self.clone())
/// }
/// ```
#[typetag::serde(tag = "type")]
pub trait Priority: Send + Sync {
    fn select(&self, queue: &[Task]) -> Option<Task>;
    fn string(&self) -> String;
    fn clone_box(&self) -> Box<dyn Priority>;
}

/// Schedules tasks in the order they were added to the queue.
#[derive(Clone, Serialize, Deserialize)]
pub struct FIFO;

#[typetag::serde]
impl Priority for FIFO {
    fn select(&self, queue: &[Task]) -> Option<Task> {
        queue.iter().find(|t| !t.completed).cloned()
    }

    fn string(&self) -> String {
        "FIFO".to_string()
    }

    fn clone_box(&self) -> Box<dyn Priority> {
        Box::new(self.clone())
    }
}

/// Schedules tasks in the order they are due.
#[derive(Clone, Serialize, Deserialize)]
pub struct Deadline;

#[typetag::serde]
impl Priority for Deadline {
    fn select(&self, queue: &[Task]) -> Option<Task> {
        queue.iter().min_by_key(|t| t.deadline).cloned()
    }

    fn string(&self) -> String {
        "Deadline".to_string()
    }

    fn clone_box(&self) -> Box<dyn Priority> {
        Box::new(self.clone())
    }
}

/// Schedules tasks in order of increasing duration: short tasks are scheduled
/// ahead of long tasks.
#[derive(Clone, Serialize, Deserialize)]
pub struct Shortest {}

#[typetag::serde]
impl Priority for Shortest {
    fn select(&self, queue: &[Task]) -> Option<Task> {
        queue.iter().min_by_key(|t| t.duration).cloned()
    }

    fn string(&self) -> String {
        "Shortest Duration".to_string()
    }

    fn clone_box(&self) -> Box<dyn Priority> {
        Box::new(self.clone())
    }
}

/// Schedules tasks in order of decreasing duration: long tasks are scheduled
/// ahead of short tasks.
#[derive(Clone, Serialize, Deserialize)]
pub struct Longest {}

#[typetag::serde]
impl Priority for Longest {
    fn select(&self, queue: &[Task]) -> Option<Task> {
        queue.iter().max_by_key(|t| t.duration).cloned()
    }

    fn string(&self) -> String {
        "Longest Duration".to_string()
    }

    fn clone_box(&self) -> Box<dyn Priority> {
        Box::new(self.clone())
    }
}

/// Schedules tasks in order of decreasing priority: higher priority tasks are
/// scheduled ahead of lower priority tasks.
#[derive(Clone, Serialize, Deserialize)]
pub struct HighestPriority {}

#[typetag::serde]
impl Priority for HighestPriority {
    fn select(&self, queue: &[Task]) -> Option<Task> {
        queue.iter().min_by_key(|t| t.priority).cloned()
    }

    fn string(&self) -> String {
        "Highest Priority".to_string()
    }

    fn clone_box(&self) -> Box<dyn Priority> {
        Box::new(self.clone())
    }
}

/// Schedules tasks in order of increasing priority: lower priority tasks are
/// scheduled ahead of higher priority tasks.
///
/// Use this priority if you hate yourself and want to feel busy.
#[derive(Clone, Serialize, Deserialize)]
pub struct LowestPriority {}

#[typetag::serde]
impl Priority for LowestPriority {
    fn select(&self, queue: &[Task]) -> Option<Task> {
        queue.iter().max_by_key(|t| t.priority).cloned()
    }

    fn string(&self) -> String {
        "Lowest Priority".to_string()
    }

    fn clone_box(&self) -> Box<dyn Priority> {
        Box::new(self.clone())
    }
}

impl Clone for Box<dyn Priority> {
    fn clone(&self) -> Self {
        self.clone_box()
    }
}

/// A score is calculated for each task based on the following formula:
/// ```rust
/// let score = (deadline_weight * (deadline - now)) - (duration_weight * duration)
/// ```
/// The lowest score gets scheduled.
///
/// This priority will attempt to schedule shorter tasks first, but will
/// schedule longer tasks if their score is lower. This allows for "urgent"
/// tasks to avoid starvation.
///
/// In the event of a tie, tasks are then scheduled by priority, then by their
/// ID.
///
/// ## Parameters
/// - `deadline_weight` - Determines the impact of deadlines on the score. A
/// higher value will effectively prioritize tasks that have closer deadlines.
/// - `duration_weight` - Determines the (negative) impact of durations on the
/// score. A higher value will prioritize tasks that have shorter durations.
#[derive(Clone, Serialize, Deserialize)]
pub struct ShortestWithUrgency {
    /// How much to prioritize deadlines over durations
    deadline_weight: i64,

    /// How much to prioritize durations over deadlines
    duration_weight: i64,
}

#[typetag::serde]
impl Priority for ShortestWithUrgency {
    fn select(&self, queue: &[Task]) -> Option<Task> {
        queue
            .iter()
            .map(|t| {
                let current_time = chrono::Local::now().naive_local();
                let deadline_distance = t.deadline - current_time;

                let score = (deadline_distance.num_seconds() / self.deadline_weight)
                    - (self.duration_weight * t.duration.num_seconds());
                println!("{score}");

                (t, score)
            })
            .min_by_key(|(t, s)| (*s, t.priority, t.id)) // score, then priority, then id
            .map(|(t, _)| t)
            .cloned()
    }

    fn string(&self) -> String {
        "Shortest Duration with Urgency".to_string()
    }

    fn clone_box(&self) -> Box<dyn Priority> {
        Box::new(self.clone())
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::{Task, TaskQueue};
    use chrono::Duration;

    #[test]
    fn test_shortest_urgency() {
        let mut queue = TaskQueue::with_priority(ShortestWithUrgency {
            deadline_weight: 1,
            duration_weight: 1,
        });

        let now = chrono::Local::now().naive_local();

        let task1 = Task::new(
            1,
            "task 1".to_string(),
            now + Duration::hours(2),
            Duration::minutes(30),
            0,
        );
        let task2 = Task::new(
            2,
            "task 2".to_string(),
            now + Duration::hours(8),
            Duration::hours(4),
            0,
        );
        let task3 = Task::new(
            3,
            "task 3".to_string(),
            now - Duration::hours(1),
            Duration::minutes(10),
            0,
        );

        queue.add(task1.clone());
        queue.add(task2.clone());
        queue.add(task3.clone());

        assert_eq!(queue.select().unwrap().id, task3.id);
        queue.delete(queue.select().unwrap().id).unwrap();
        assert_eq!(queue.select().unwrap().id, task1.id);
        queue.delete(queue.select().unwrap().id).unwrap();
        assert_eq!(queue.select().unwrap().id, task2.id);
        queue.delete(queue.select().unwrap().id).unwrap();

        drop(queue);
    }

    #[test]
    fn test_shortest_urgency_tie() {
        let mut queue = TaskQueue::with_priority(ShortestWithUrgency {
            duration_weight: 1,
            deadline_weight: 1,
        });

        let now = chrono::Local::now().naive_local();

        let task1 = Task::new(
            1,
            "task 1".to_string(),
            now + Duration::hours(4),
            Duration::hours(4),
            1,
        );
        let task2 = Task::new(
            2,
            "task 2".to_string(),
            now + Duration::hours(8),
            Duration::hours(8),
            5,
        );

        queue.add(task1.clone());
        queue.add(task2.clone());

        assert_eq!(queue.select().unwrap().id, task1.id);
        queue.delete(queue.select().unwrap().id).unwrap();
        assert_eq!(queue.select().unwrap().id, task2.id);
        queue.delete(queue.select().unwrap().id).unwrap();
    }
}
