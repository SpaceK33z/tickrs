use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

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
}
