//! Project API endpoints for TickTick

use crate::api::client::{ApiError, TickTickClient};
use crate::models::{Project, ProjectData, INBOX_PROJECT_ID};
use tracing::{debug, instrument};

/// Request body for creating a project
#[derive(Debug, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateProjectRequest {
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub color: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub view_mode: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub kind: Option<String>,
}

/// Request body for updating a project
#[derive(Debug, serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateProjectRequest {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub color: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub closed: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub view_mode: Option<String>,
}

impl TickTickClient {
    /// List all projects
    ///
    /// GET /project
    /// Note: Appends INBOX project to results since it's not returned by the API
    #[instrument(skip(self))]
    pub async fn list_projects(&self) -> Result<Vec<Project>, ApiError> {
        debug!("Listing all projects");

        let mut projects: Vec<Project> = self.get("/project").await?;

        // Add INBOX project at the beginning (it's not returned by the API)
        projects.insert(0, Project::inbox());

        debug!("Found {} projects (including inbox)", projects.len());
        Ok(projects)
    }

    /// Get a single project by ID
    ///
    /// GET /project/{id}
    #[instrument(skip(self))]
    pub async fn get_project(&self, id: &str) -> Result<Project, ApiError> {
        debug!("Getting project: {}", id);

        // Handle special INBOX case
        if id == INBOX_PROJECT_ID {
            return Ok(Project::inbox());
        }

        let endpoint = format!("/project/{}", id);
        self.get(&endpoint).await
    }

    /// Get project with its tasks and columns
    ///
    /// GET /project/{id}/data
    #[instrument(skip(self))]
    pub async fn get_project_data(&self, id: &str) -> Result<ProjectData, ApiError> {
        debug!("Getting project data: {}", id);

        let endpoint = format!("/project/{}/data", id);
        self.get(&endpoint).await
    }

    /// Create a new project
    ///
    /// POST /project
    #[instrument(skip(self))]
    pub async fn create_project(
        &self,
        request: &CreateProjectRequest,
    ) -> Result<Project, ApiError> {
        debug!("Creating project: {}", request.name);

        self.post("/project", request).await
    }

    /// Update an existing project
    ///
    /// POST /project/{id}
    #[instrument(skip(self))]
    pub async fn update_project(
        &self,
        id: &str,
        request: &UpdateProjectRequest,
    ) -> Result<Project, ApiError> {
        debug!("Updating project: {}", id);

        if id == INBOX_PROJECT_ID {
            return Err(ApiError::BadRequest(
                "Cannot update INBOX project".to_string(),
            ));
        }

        let endpoint = format!("/project/{}", id);
        self.post(&endpoint, request).await
    }

    /// Delete a project
    ///
    /// DELETE /project/{id}
    #[instrument(skip(self))]
    pub async fn delete_project(&self, id: &str) -> Result<(), ApiError> {
        debug!("Deleting project: {}", id);

        if id == INBOX_PROJECT_ID {
            return Err(ApiError::BadRequest(
                "Cannot delete INBOX project".to_string(),
            ));
        }

        let endpoint = format!("/project/{}", id);
        self.delete(&endpoint).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_project_request_serialization() {
        let request = CreateProjectRequest {
            name: "Test Project".to_string(),
            color: Some("#FF5733".to_string()),
            view_mode: Some("list".to_string()),
            kind: None,
        };

        let json = serde_json::to_string(&request).unwrap();
        assert!(json.contains("\"name\":\"Test Project\""));
        assert!(json.contains("\"color\":\"#FF5733\""));
        assert!(json.contains("\"viewMode\":\"list\""));
        assert!(!json.contains("kind")); // Should be skipped when None
    }

    #[test]
    fn test_update_project_request_serialization() {
        let request = UpdateProjectRequest {
            name: Some("Updated Name".to_string()),
            color: None,
            closed: Some(true),
            view_mode: None,
        };

        let json = serde_json::to_string(&request).unwrap();
        assert!(json.contains("\"name\":\"Updated Name\""));
        assert!(json.contains("\"closed\":true"));
        assert!(!json.contains("color")); // Should be skipped when None
        assert!(!json.contains("viewMode")); // Should be skipped when None
    }
}
