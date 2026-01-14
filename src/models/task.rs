use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use super::{ChecklistItem, ChecklistItemRequest, Priority, Status};

/// Task model matching TickTick API format
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Task {
    pub id: String,
    pub project_id: String,
    pub title: String,
    #[serde(default)]
    pub is_all_day: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub completed_time: Option<DateTime<Utc>>,
    #[serde(default)]
    pub content: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub due_date: Option<DateTime<Utc>>,
    #[serde(default)]
    pub items: Vec<ChecklistItem>,
    #[serde(default)]
    pub priority: Priority,
    #[serde(default)]
    pub reminders: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub repeat_flag: Option<String>,
    #[serde(default)]
    pub sort_order: i64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub start_date: Option<DateTime<Utc>>,
    #[serde(default)]
    pub status: Status,
    #[serde(default)]
    pub time_zone: String,
    #[serde(default)]
    pub tags: Vec<String>,
}

impl Task {
    /// Check if the task is complete
    pub fn is_complete(&self) -> bool {
        self.status.is_complete()
    }
}

/// Builder for creating new [`Task`] instances with a fluent API.
///
/// Use this builder when you need to construct a task programmatically
/// with optional fields. The builder provides sensible defaults for
/// all optional fields.
///
/// # Required Fields
///
/// - `project_id` - The project to add the task to
/// - `title` - The task title
///
/// # Example
///
/// ```
/// use tickrs::models::task::TaskBuilder;
/// use tickrs::models::{Priority, ChecklistItemRequest};
///
/// let task = TaskBuilder::new("proj123", "Complete documentation")
///     .content("Add doc comments to all public APIs")
///     .priority(Priority::High)
///     .tags(vec!["work".to_string(), "docs".to_string()])
///     .build();
///
/// assert_eq!(task.title, "Complete documentation");
/// assert_eq!(task.priority, Priority::High);
///
/// // Create a task with subtasks
/// let task_with_subtasks = TaskBuilder::new("proj123", "Pack for trip")
///     .items(vec![
///         ChecklistItemRequest::new("Passport"),
///         ChecklistItemRequest::new("Clothes"),
///     ])
///     .build();
/// ```
#[derive(Default)]
#[allow(dead_code)] // Available for external use; tested in tests
pub struct TaskBuilder {
    project_id: String,
    title: String,
    is_all_day: bool,
    content: Option<String>,
    due_date: Option<DateTime<Utc>>,
    priority: Priority,
    start_date: Option<DateTime<Utc>>,
    time_zone: Option<String>,
    tags: Vec<String>,
    items: Vec<ChecklistItemRequest>,
}

#[allow(dead_code)] // Builder methods available for external use; tested
impl TaskBuilder {
    /// Create a new TaskBuilder with required fields.
    ///
    /// # Arguments
    ///
    /// * `project_id` - The ID of the project to add the task to
    /// * `title` - The task title
    pub fn new(project_id: impl Into<String>, title: impl Into<String>) -> Self {
        Self {
            project_id: project_id.into(),
            title: title.into(),
            ..Default::default()
        }
    }

    /// Set the task description/notes.
    pub fn content(mut self, content: impl Into<String>) -> Self {
        self.content = Some(content.into());
        self
    }

    /// Set the task priority level.
    pub fn priority(mut self, priority: Priority) -> Self {
        self.priority = priority;
        self
    }

    /// Set the task due date.
    pub fn due_date(mut self, due: DateTime<Utc>) -> Self {
        self.due_date = Some(due);
        self
    }

    /// Set the task start date.
    pub fn start_date(mut self, start: DateTime<Utc>) -> Self {
        self.start_date = Some(start);
        self
    }

    /// Set whether this is an all-day task (no specific time).
    pub fn all_day(mut self, is_all_day: bool) -> Self {
        self.is_all_day = is_all_day;
        self
    }

    /// Set the task timezone (IANA format, e.g., "America/New_York").
    pub fn time_zone(mut self, tz: impl Into<String>) -> Self {
        self.time_zone = Some(tz.into());
        self
    }

    /// Set the task tags.
    pub fn tags(mut self, tags: Vec<String>) -> Self {
        self.tags = tags;
        self
    }

    /// Set the task's subtasks/checklist items.
    ///
    /// # Example
    ///
    /// ```
    /// use tickrs::models::task::TaskBuilder;
    /// use tickrs::models::ChecklistItemRequest;
    ///
    /// let task = TaskBuilder::new("proj123", "Shopping list")
    ///     .items(vec![
    ///         ChecklistItemRequest::new("Milk"),
    ///         ChecklistItemRequest::new("Bread").with_sort_order(1),
    ///         ChecklistItemRequest::new("Eggs").completed(),
    ///     ])
    ///     .build();
    /// ```
    pub fn items(mut self, items: Vec<ChecklistItemRequest>) -> Self {
        self.items = items;
        self
    }

    /// Build the [`Task`] instance.
    ///
    /// The returned task will have an empty `id` field, which will be
    /// populated by the API when the task is created.
    ///
    /// Note: Subtasks set via [`items()`](Self::items) are not included in the
    /// built Task. Use [`into_create_request()`](Self::into_create_request) to
    /// create a request that includes subtasks for the API.
    pub fn build(self) -> Task {
        Task {
            id: String::new(), // Will be set by API
            project_id: self.project_id,
            title: self.title,
            is_all_day: self.is_all_day,
            completed_time: None,
            content: self.content.unwrap_or_default(),
            due_date: self.due_date,
            items: Vec::new(),
            priority: self.priority,
            reminders: Vec::new(),
            repeat_flag: None,
            sort_order: 0,
            start_date: self.start_date,
            status: Status::Normal,
            time_zone: self.time_zone.unwrap_or_default(),
            tags: self.tags,
        }
    }

    /// Build a [`CreateTaskRequest`](crate::api::CreateTaskRequest) for the API.
    ///
    /// This method creates a request that can be passed to
    /// [`TickTickClient::create_task()`](crate::api::TickTickClient::create_task).
    /// Unlike [`build()`](Self::build), this includes subtasks set via
    /// [`items()`](Self::items).
    ///
    /// # Example
    ///
    /// ```
    /// use tickrs::models::task::TaskBuilder;
    /// use tickrs::models::ChecklistItemRequest;
    ///
    /// let request = TaskBuilder::new("proj123", "Pack for trip")
    ///     .items(vec![
    ///         ChecklistItemRequest::new("Passport"),
    ///         ChecklistItemRequest::new("Clothes"),
    ///     ])
    ///     .into_create_request();
    ///
    /// // Now use: client.create_task(&request).await
    /// ```
    pub fn into_create_request(self) -> crate::api::CreateTaskRequest {
        crate::api::CreateTaskRequest {
            title: self.title,
            project_id: self.project_id,
            content: self.content,
            is_all_day: if self.is_all_day { Some(true) } else { None },
            start_date: self.start_date.map(|d| d.to_rfc3339()),
            due_date: self.due_date.map(|d| d.to_rfc3339()),
            priority: if self.priority != Priority::None {
                Some(self.priority.to_api_value())
            } else {
                None
            },
            time_zone: self.time_zone,
            tags: if self.tags.is_empty() {
                None
            } else {
                Some(self.tags)
            },
            items: if self.items.is_empty() {
                None
            } else {
                Some(self.items)
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_task_deserialization() {
        let json = r#"{
            "id": "task123",
            "projectId": "proj456",
            "title": "Test Task",
            "isAllDay": false,
            "content": "Task description",
            "priority": 3,
            "status": 0,
            "tags": ["work", "urgent"],
            "items": [],
            "reminders": [],
            "sortOrder": 0,
            "timeZone": "UTC"
        }"#;

        let task: Task = serde_json::from_str(json).unwrap();
        assert_eq!(task.id, "task123");
        assert_eq!(task.project_id, "proj456");
        assert_eq!(task.title, "Test Task");
        assert_eq!(task.priority, Priority::Medium);
        assert!(!task.is_complete());
        assert_eq!(task.tags, vec!["work", "urgent"]);
    }

    #[test]
    fn test_task_builder() {
        let task = TaskBuilder::new("proj123", "New Task")
            .content("Description")
            .priority(Priority::High)
            .all_day(true)
            .tags(vec!["test".to_string()])
            .build();

        assert_eq!(task.project_id, "proj123");
        assert_eq!(task.title, "New Task");
        assert_eq!(task.content, "Description");
        assert_eq!(task.priority, Priority::High);
        assert!(task.is_all_day);
        assert_eq!(task.tags, vec!["test"]);
    }

    #[test]
    fn test_task_special_characters_in_title() {
        let json = r#"{
            "id": "task789",
            "projectId": "proj456",
            "title": "Test <script>alert('xss')</script> & \"quotes\" 'apostrophes' émojis 🎉",
            "isAllDay": false,
            "content": "",
            "priority": 0,
            "status": 0,
            "tags": [],
            "items": [],
            "reminders": [],
            "sortOrder": 0,
            "timeZone": ""
        }"#;

        let task: Task = serde_json::from_str(json).unwrap();
        assert_eq!(
            task.title,
            "Test <script>alert('xss')</script> & \"quotes\" 'apostrophes' émojis 🎉"
        );

        // Verify round-trip serialization
        let serialized = serde_json::to_string(&task).unwrap();
        let task2: Task = serde_json::from_str(&serialized).unwrap();
        assert_eq!(task.title, task2.title);
    }

    #[test]
    fn test_task_with_subtasks() {
        let json = r#"{
            "id": "task123",
            "projectId": "proj456",
            "title": "Task with subtasks",
            "isAllDay": false,
            "content": "",
            "priority": 0,
            "status": 0,
            "tags": [],
            "items": [
                {"id": "sub1", "title": "Subtask 1", "status": 0, "completedTime": 0, "isAllDay": false, "sortOrder": 0, "timeZone": ""},
                {"id": "sub2", "title": "Subtask 2", "status": 1, "completedTime": 1704067200, "isAllDay": false, "sortOrder": 1, "timeZone": ""}
            ],
            "reminders": [],
            "sortOrder": 0,
            "timeZone": ""
        }"#;

        let task: Task = serde_json::from_str(json).unwrap();
        assert_eq!(task.items.len(), 2);
        assert_eq!(task.items[0].title, "Subtask 1");
        assert!(!task.items[0].is_complete());
        assert_eq!(task.items[1].title, "Subtask 2");
        assert!(task.items[1].is_complete());
    }

    #[test]
    fn test_task_with_dates() {
        let json = r#"{
            "id": "task123",
            "projectId": "proj456",
            "title": "Task with dates",
            "isAllDay": true,
            "content": "",
            "dueDate": "2026-01-15T14:00:00Z",
            "startDate": "2026-01-10T09:00:00Z",
            "priority": 5,
            "status": 0,
            "tags": [],
            "items": [],
            "reminders": [],
            "sortOrder": 0,
            "timeZone": "America/New_York"
        }"#;

        let task: Task = serde_json::from_str(json).unwrap();
        assert!(task.is_all_day);
        assert!(task.due_date.is_some());
        assert!(task.start_date.is_some());
        assert_eq!(task.time_zone, "America/New_York");
        assert_eq!(task.priority, Priority::High);
    }

    #[test]
    fn test_task_completed() {
        let json = r#"{
            "id": "task123",
            "projectId": "proj456",
            "title": "Completed task",
            "isAllDay": false,
            "completedTime": "2026-01-14T10:30:00Z",
            "content": "",
            "priority": 0,
            "status": 2,
            "tags": [],
            "items": [],
            "reminders": [],
            "sortOrder": 0,
            "timeZone": ""
        }"#;

        let task: Task = serde_json::from_str(json).unwrap();
        assert!(task.is_complete());
        assert!(task.completed_time.is_some());
        assert_eq!(task.status, Status::Complete);
    }

    #[test]
    fn test_task_minimal_json() {
        // Test deserializing a task with only required fields
        let json = r#"{
            "id": "task123",
            "projectId": "proj456",
            "title": "Minimal task"
        }"#;

        let task: Task = serde_json::from_str(json).unwrap();
        assert_eq!(task.id, "task123");
        assert_eq!(task.project_id, "proj456");
        assert_eq!(task.title, "Minimal task");
        assert!(!task.is_all_day);
        assert!(task.items.is_empty());
        assert!(task.tags.is_empty());
        assert_eq!(task.priority, Priority::None);
        assert_eq!(task.status, Status::Normal);
    }
}
