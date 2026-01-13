use serde::{Deserialize, Serialize};

use super::Task;

/// Special INBOX project ID
pub const INBOX_PROJECT_ID: &str = "inbox";

/// Project model matching TickTick API format
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Project {
    pub id: String,
    pub name: String,
    #[serde(default)]
    pub color: String,
    #[serde(default)]
    pub sort_order: i64,
    #[serde(default)]
    pub closed: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub group_id: Option<String>,
    #[serde(default = "default_view_mode")]
    pub view_mode: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub permission: Option<String>,
    #[serde(default = "default_kind")]
    pub kind: String,
}

fn default_view_mode() -> String {
    "list".to_string()
}

fn default_kind() -> String {
    "TASK".to_string()
}

impl Project {
    /// Create a representation of the special INBOX project
    pub fn inbox() -> Self {
        Self {
            id: INBOX_PROJECT_ID.to_string(),
            name: "Inbox".to_string(),
            color: String::new(),
            sort_order: -1,
            closed: false,
            group_id: None,
            view_mode: "list".to_string(),
            permission: None,
            kind: "TASK".to_string(),
        }
    }

    /// Check if this is the INBOX project
    pub fn is_inbox(&self) -> bool {
        self.id == INBOX_PROJECT_ID
    }
}

/// Project with its tasks and columns (kanban)
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProjectData {
    pub project: Project,
    #[serde(default)]
    pub tasks: Vec<Task>,
    #[serde(default)]
    pub columns: Vec<Column>,
}

/// Kanban column
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Column {
    pub id: String,
    pub name: String,
    #[serde(default)]
    pub sort_order: i64,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_project_deserialization() {
        let json = "{\"id\":\"proj123\",\"name\":\"Work\",\"color\":\"#FF5733\",\"sortOrder\":0,\"closed\":false,\"viewMode\":\"list\",\"kind\":\"TASK\"}";

        let project: Project = serde_json::from_str(json).unwrap();
        assert_eq!(project.id, "proj123");
        assert_eq!(project.name, "Work");
        assert_eq!(project.color, "#FF5733");
        assert!(!project.is_inbox());
    }

    #[test]
    fn test_inbox_project() {
        let inbox = Project::inbox();
        assert!(inbox.is_inbox());
        assert_eq!(inbox.id, "inbox");
        assert_eq!(inbox.name, "Inbox");
    }

    #[test]
    fn test_project_data_deserialization() {
        let json = "{\"project\":{\"id\":\"proj123\",\"name\":\"Work\",\"color\":\"#FF5733\",\"sortOrder\":0,\"closed\":false,\"viewMode\":\"list\",\"kind\":\"TASK\"},\"tasks\":[],\"columns\":[]}";

        let data: ProjectData = serde_json::from_str(json).unwrap();
        assert_eq!(data.project.id, "proj123");
        assert!(data.tasks.is_empty());
    }
}
