//! Task API endpoints for TickTick

use crate::api::client::{ApiError, TickTickClient};
use crate::models::{Task, Status};
use tracing::{debug, instrument};

/// Request body for creating a task
#[derive(Debug, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateTaskRequest {
    pub title: String,
    pub project_id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub content: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub is_all_day: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub start_date: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub due_date: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub priority: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub time_zone: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tags: Option<Vec<String>>,
}

/// Request body for updating a task
#[derive(Debug, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateTaskRequest {
    /// Task ID (required for update)
    pub id: String,
    /// Project ID (required for update)
    pub project_id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub content: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub is_all_day: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub start_date: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub due_date: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub priority: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub time_zone: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tags: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub status: Option<i32>,
}

impl TickTickClient {
    /// List all tasks in a project
    ///
    /// Uses GET /project/{projectId}/data and extracts tasks
    #[instrument(skip(self))]
    pub async fn list_tasks(&self, project_id: &str) -> Result<Vec<Task>, ApiError> {
        debug!("Listing tasks for project: {}", project_id);

        let project_data = self.get_project_data(project_id).await?;

        debug!("Found {} tasks", project_data.tasks.len());
        Ok(project_data.tasks)
    }

    /// Get a single task by ID
    ///
    /// GET /project/{projectId}/task/{taskId}
    #[instrument(skip(self))]
    pub async fn get_task(&self, project_id: &str, task_id: &str) -> Result<Task, ApiError> {
        debug!("Getting task: {} in project: {}", task_id, project_id);

        let endpoint = format!("/project/{}/task/{}", project_id, task_id);
        self.get(&endpoint).await
    }

    /// Create a new task
    ///
    /// POST /task
    #[instrument(skip(self))]
    pub async fn create_task(&self, request: &CreateTaskRequest) -> Result<Task, ApiError> {
        debug!("Creating task: {} in project: {}", request.title, request.project_id);

        self.post("/task", request).await
    }

    /// Update an existing task
    ///
    /// POST /task/{id}
    #[instrument(skip(self))]
    pub async fn update_task(&self, task_id: &str, request: &UpdateTaskRequest) -> Result<Task, ApiError> {
        debug!("Updating task: {}", task_id);

        let endpoint = format!("/task/{}", task_id);
        self.post(&endpoint, request).await
    }

    /// Delete a task
    ///
    /// DELETE /project/{projectId}/task/{taskId}
    #[instrument(skip(self))]
    pub async fn delete_task(&self, project_id: &str, task_id: &str) -> Result<(), ApiError> {
        debug!("Deleting task: {} from project: {}", task_id, project_id);

        let endpoint = format!("/project/{}/task/{}", project_id, task_id);
        self.delete(&endpoint).await
    }

    /// Mark a task as complete
    ///
    /// POST /project/{projectId}/task/{taskId}/complete
    #[instrument(skip(self))]
    pub async fn complete_task(&self, project_id: &str, task_id: &str) -> Result<(), ApiError> {
        debug!("Completing task: {} in project: {}", task_id, project_id);

        let endpoint = format!("/project/{}/task/{}/complete", project_id, task_id);
        // The complete endpoint returns empty body on success
        let _: serde_json::Value = self.post_empty(&endpoint).await?;
        Ok(())
    }

    /// Mark a task as incomplete (uncomplete)
    ///
    /// Updates task status to 0 (Normal)
    #[instrument(skip(self))]
    pub async fn uncomplete_task(&self, project_id: &str, task_id: &str) -> Result<Task, ApiError> {
        debug!("Uncompleting task: {} in project: {}", task_id, project_id);

        let request = UpdateTaskRequest {
            id: task_id.to_string(),
            project_id: project_id.to_string(),
            title: None,
            content: None,
            is_all_day: None,
            start_date: None,
            due_date: None,
            priority: None,
            time_zone: None,
            tags: None,
            status: Some(Status::Normal.to_api_value()),
        };

        self.update_task(task_id, &request).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_task_request_serialization() {
        let request = CreateTaskRequest {
            title: "Test Task".to_string(),
            project_id: "proj123".to_string(),
            content: Some("Description".to_string()),
            is_all_day: Some(false),
            start_date: None,
            due_date: Some("2026-01-15T14:00:00+0000".to_string()),
            priority: Some(3),
            time_zone: Some("UTC".to_string()),
            tags: Some(vec!["work".to_string(), "urgent".to_string()]),
        };

        let json = serde_json::to_string(&request).unwrap();
        assert!(json.contains("\"title\":\"Test Task\""));
        assert!(json.contains("\"projectId\":\"proj123\""));
        assert!(json.contains("\"content\":\"Description\""));
        assert!(json.contains("\"dueDate\":\"2026-01-15T14:00:00+0000\""));
        assert!(json.contains("\"priority\":3"));
        assert!(!json.contains("startDate")); // Should be skipped when None
    }

    #[test]
    fn test_create_task_request_minimal() {
        let request = CreateTaskRequest {
            title: "Minimal Task".to_string(),
            project_id: "proj123".to_string(),
            content: None,
            is_all_day: None,
            start_date: None,
            due_date: None,
            priority: None,
            time_zone: None,
            tags: None,
        };

        let json = serde_json::to_string(&request).unwrap();
        assert!(json.contains("\"title\":\"Minimal Task\""));
        assert!(json.contains("\"projectId\":\"proj123\""));
        // Only required fields should be present
        assert!(!json.contains("content"));
        assert!(!json.contains("priority"));
    }

    #[test]
    fn test_update_task_request_serialization() {
        let request = UpdateTaskRequest {
            id: "task123".to_string(),
            project_id: "proj456".to_string(),
            title: Some("Updated Title".to_string()),
            content: None,
            is_all_day: None,
            start_date: None,
            due_date: None,
            priority: Some(5),
            time_zone: None,
            tags: None,
            status: None,
        };

        let json = serde_json::to_string(&request).unwrap();
        assert!(json.contains("\"id\":\"task123\""));
        assert!(json.contains("\"projectId\":\"proj456\""));
        assert!(json.contains("\"title\":\"Updated Title\""));
        assert!(json.contains("\"priority\":5"));
        assert!(!json.contains("content")); // Should be skipped when None
        assert!(!json.contains("status")); // Should be skipped when None
    }

    #[test]
    fn test_update_task_request_status_change() {
        let request = UpdateTaskRequest {
            id: "task123".to_string(),
            project_id: "proj456".to_string(),
            title: None,
            content: None,
            is_all_day: None,
            start_date: None,
            due_date: None,
            priority: None,
            time_zone: None,
            tags: None,
            status: Some(0), // Normal/incomplete
        };

        let json = serde_json::to_string(&request).unwrap();
        assert!(json.contains("\"status\":0"));
    }
}
