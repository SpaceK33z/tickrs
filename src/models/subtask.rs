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
}
