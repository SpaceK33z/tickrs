use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// Request body for creating or updating a checklist item (subtask).
///
/// Use this when creating or updating tasks with subtasks via
/// [`CreateTaskRequest`](crate::api::CreateTaskRequest) or
/// [`UpdateTaskRequest`](crate::api::UpdateTaskRequest).
///
/// # Example
///
/// ```
/// use ticktickrs::models::ChecklistItemRequest;
///
/// let subtasks = vec![
///     ChecklistItemRequest::new("Pack passport"),
///     ChecklistItemRequest::new("Book hotel").completed(),
///     ChecklistItemRequest::new("Confirm flight").with_sort_order(2),
/// ];
/// ```
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ChecklistItemRequest {
    /// Subtask title (required)
    pub title: String,
    /// Completion status: 0 (incomplete), 1+ (complete)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub status: Option<i32>,
    /// Sort order for display (lower values appear first)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sort_order: Option<i64>,
}

impl ChecklistItemRequest {
    /// Create a new subtask request with the given title.
    pub fn new(title: impl Into<String>) -> Self {
        Self {
            title: title.into(),
            status: None,
            sort_order: None,
        }
    }

    /// Mark this subtask as completed.
    #[allow(dead_code)]
    pub fn completed(mut self) -> Self {
        self.status = Some(1);
        self
    }

    /// Set the sort order for this subtask.
    pub fn with_sort_order(mut self, order: i64) -> Self {
        self.sort_order = Some(order);
        self
    }
}

/// Checklist item (subtask) within a task
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ChecklistItem {
    pub id: String,
    pub title: String,
    pub status: i32,
    #[serde(default)]
    pub completed_time: i64,
    #[serde(default)]
    pub is_all_day: bool,
    #[serde(default)]
    pub sort_order: i64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub start_date: Option<DateTime<Utc>>,
    #[serde(default)]
    pub time_zone: String,
}

impl ChecklistItem {
    /// Check if the subtask is complete
    pub fn is_complete(&self) -> bool {
        self.status != 0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_checklist_item_deserialization() {
        let json = r#"{
            "id": "item123",
            "title": "Subtask 1",
            "status": 0,
            "completedTime": 0,
            "isAllDay": false,
            "sortOrder": 0,
            "timeZone": "UTC"
        }"#;

        let item: ChecklistItem = serde_json::from_str(json).unwrap();
        assert_eq!(item.id, "item123");
        assert_eq!(item.title, "Subtask 1");
        assert!(!item.is_complete());
    }

    #[test]
    fn test_checklist_item_complete() {
        let json = r#"{
            "id": "item456",
            "title": "Done task",
            "status": 1,
            "completedTime": 1704067200,
            "isAllDay": false,
            "sortOrder": 1,
            "timeZone": "America/New_York"
        }"#;

        let item: ChecklistItem = serde_json::from_str(json).unwrap();
        assert!(item.is_complete());
    }

    #[test]
    fn test_checklist_item_special_characters() {
        let json = r#"{
            "id": "item789",
            "title": "Buy groceries: milk & eggs \"fresh\" <organic>",
            "status": 0,
            "completedTime": 0,
            "isAllDay": false,
            "sortOrder": 0,
            "timeZone": ""
        }"#;

        let item: ChecklistItem = serde_json::from_str(json).unwrap();
        assert_eq!(item.title, "Buy groceries: milk & eggs \"fresh\" <organic>");

        // Verify round-trip
        let serialized = serde_json::to_string(&item).unwrap();
        let item2: ChecklistItem = serde_json::from_str(&serialized).unwrap();
        assert_eq!(item.title, item2.title);
    }

    #[test]
    fn test_checklist_item_minimal_json() {
        let json = r#"{
            "id": "item123",
            "title": "Minimal item",
            "status": 0
        }"#;

        let item: ChecklistItem = serde_json::from_str(json).unwrap();
        assert_eq!(item.id, "item123");
        assert_eq!(item.title, "Minimal item");
        assert!(!item.is_complete());
        assert_eq!(item.completed_time, 0);
        assert!(!item.is_all_day);
        assert_eq!(item.sort_order, 0);
        assert!(item.time_zone.is_empty());
    }

    #[test]
    fn test_checklist_item_with_start_date() {
        let json = r#"{
            "id": "item123",
            "title": "Scheduled item",
            "status": 0,
            "completedTime": 0,
            "isAllDay": true,
            "sortOrder": 0,
            "startDate": "2026-01-20T00:00:00Z",
            "timeZone": "Europe/London"
        }"#;

        let item: ChecklistItem = serde_json::from_str(json).unwrap();
        assert!(item.is_all_day);
        assert!(item.start_date.is_some());
        assert_eq!(item.time_zone, "Europe/London");
    }

    #[test]
    fn test_checklist_item_request_serialization() {
        let request = ChecklistItemRequest::new("Pack passport");
        let json = serde_json::to_string(&request).unwrap();

        assert!(json.contains("\"title\":\"Pack passport\""));
        // status and sortOrder should be omitted when None
        assert!(!json.contains("status"));
        assert!(!json.contains("sortOrder"));
    }

    #[test]
    fn test_checklist_item_request_completed() {
        let request = ChecklistItemRequest::new("Done item").completed();
        let json = serde_json::to_string(&request).unwrap();

        assert!(json.contains("\"title\":\"Done item\""));
        assert!(json.contains("\"status\":1"));
    }

    #[test]
    fn test_checklist_item_request_with_sort_order() {
        let request = ChecklistItemRequest::new("Ordered item").with_sort_order(5);
        let json = serde_json::to_string(&request).unwrap();

        assert!(json.contains("\"title\":\"Ordered item\""));
        assert!(json.contains("\"sortOrder\":5"));
    }

    #[test]
    fn test_checklist_item_request_full() {
        let request = ChecklistItemRequest::new("Full item")
            .completed()
            .with_sort_order(10);
        let json = serde_json::to_string(&request).unwrap();

        assert!(json.contains("\"title\":\"Full item\""));
        assert!(json.contains("\"status\":1"));
        assert!(json.contains("\"sortOrder\":10"));
    }
}
