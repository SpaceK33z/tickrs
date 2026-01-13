use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use super::{ChecklistItem, Priority, Status};

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

/// Builder for creating new tasks
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
}

#[allow(dead_code)] // Builder methods available for external use; tested
impl TaskBuilder {
    pub fn new(project_id: impl Into<String>, title: impl Into<String>) -> Self {
        Self {
            project_id: project_id.into(),
            title: title.into(),
            ..Default::default()
        }
    }

    pub fn content(mut self, content: impl Into<String>) -> Self {
        self.content = Some(content.into());
        self
    }

    pub fn priority(mut self, priority: Priority) -> Self {
        self.priority = priority;
        self
    }

    pub fn due_date(mut self, due: DateTime<Utc>) -> Self {
        self.due_date = Some(due);
        self
    }

    pub fn start_date(mut self, start: DateTime<Utc>) -> Self {
        self.start_date = Some(start);
        self
    }

    pub fn all_day(mut self, is_all_day: bool) -> Self {
        self.is_all_day = is_all_day;
        self
    }

    pub fn time_zone(mut self, tz: impl Into<String>) -> Self {
        self.time_zone = Some(tz.into());
        self
    }

    pub fn tags(mut self, tags: Vec<String>) -> Self {
        self.tags = tags;
        self
    }

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
}
