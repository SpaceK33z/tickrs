use serde::{Deserialize, Serialize};

/// Standard JSON response wrapper for all commands
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JsonResponse<T> {
    /// Whether the operation was successful
    pub success: bool,
    /// The data payload (present on success)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<T>,
    /// Error details (present on failure)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<ErrorDetail>,
    /// Human-readable message
    #[serde(skip_serializing_if = "Option::is_none")]
    pub message: Option<String>,
}

/// Error details for JSON error responses
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ErrorDetail {
    /// Machine-readable error code
    pub code: String,
    /// Human-readable error message
    pub message: String,
    /// Additional context (optional)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub details: Option<serde_json::Value>,
}

impl<T: Serialize> JsonResponse<T> {
    /// Create a success response with data
    pub fn success(data: T) -> Self {
        Self {
            success: true,
            data: Some(data),
            error: None,
            message: None,
        }
    }

    /// Create a success response with data and message
    pub fn success_with_message(data: T, message: impl Into<String>) -> Self {
        Self {
            success: true,
            data: Some(data),
            error: None,
            message: Some(message.into()),
        }
    }
}

impl<T> JsonResponse<T> {
    /// Create an error response
    #[allow(dead_code)] // Available for external use
    pub fn error(code: impl Into<String>, message: impl Into<String>) -> Self {
        Self {
            success: false,
            data: None,
            error: Some(ErrorDetail {
                code: code.into(),
                message: message.into(),
                details: None,
            }),
            message: None,
        }
    }

    /// Create an error response with additional details
    #[allow(dead_code)] // Available for external use
    pub fn error_with_details(
        code: impl Into<String>,
        message: impl Into<String>,
        details: serde_json::Value,
    ) -> Self {
        Self {
            success: false,
            data: None,
            error: Some(ErrorDetail {
                code: code.into(),
                message: message.into(),
                details: Some(details),
            }),
            message: None,
        }
    }
}

impl<T: Serialize> JsonResponse<T> {
    /// Convert response to JSON string with pretty printing
    pub fn to_json_string(&self) -> String {
        serde_json::to_string_pretty(self).unwrap_or_else(|e| {
            format!(
                r#"{{"success":false,"error":{{"code":"SERIALIZATION_ERROR","message":"{}"}}}}"#,
                e
            )
        })
    }
}

/// Convert a Result to a JSON response string
#[allow(dead_code)] // Available for external use
pub fn result_to_json<T: Serialize, E: std::fmt::Display>(result: Result<T, E>) -> String {
    match result {
        Ok(data) => JsonResponse::success(data).to_json_string(),
        Err(e) => JsonResponse::<()>::error("ERROR", e.to_string()).to_json_string(),
    }
}

/// Convert a Result to a JSON response string with a success message
#[allow(dead_code)] // Available for external use
pub fn result_to_json_with_message<T: Serialize, E: std::fmt::Display>(
    result: Result<T, E>,
    message: &str,
) -> String {
    match result {
        Ok(data) => JsonResponse::success_with_message(data, message).to_json_string(),
        Err(e) => JsonResponse::<()>::error("ERROR", e.to_string()).to_json_string(),
    }
}

/// Data wrapper for project list output
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectListData {
    pub projects: Vec<crate::models::Project>,
}

/// Data wrapper for single project output
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectData {
    pub project: crate::models::Project,
}

/// Data wrapper for task list output
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskListData {
    pub tasks: Vec<crate::models::Task>,
    pub count: usize,
}

/// Data wrapper for single task output
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskData {
    pub task: crate::models::Task,
}

/// Data wrapper for subtask list output
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SubtaskListData {
    pub subtasks: Vec<crate::models::ChecklistItem>,
    pub count: usize,
}

/// Data wrapper for version output
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VersionData {
    pub version: String,
    pub name: String,
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_success_response() {
        let response = JsonResponse::success(json!({"id": "123"}));
        assert!(response.success);
        assert!(response.data.is_some());
        assert!(response.error.is_none());
    }

    #[test]
    fn test_success_with_message() {
        let response =
            JsonResponse::success_with_message(json!({"id": "123"}), "Task created successfully");
        assert!(response.success);
        assert_eq!(
            response.message,
            Some("Task created successfully".to_string())
        );
    }

    #[test]
    fn test_error_response() {
        let response: JsonResponse<()> = JsonResponse::error("NOT_FOUND", "Task not found");
        assert!(!response.success);
        assert!(response.data.is_none());
        let error = response.error.unwrap();
        assert_eq!(error.code, "NOT_FOUND");
        assert_eq!(error.message, "Task not found");
    }

    #[test]
    fn test_error_with_details() {
        let response: JsonResponse<()> = JsonResponse::error_with_details(
            "VALIDATION_ERROR",
            "Invalid input",
            json!({"field": "title", "reason": "cannot be empty"}),
        );
        assert!(!response.success);
        let error = response.error.unwrap();
        assert_eq!(error.code, "VALIDATION_ERROR");
        assert!(error.details.is_some());
    }

    #[test]
    fn test_to_json_string() {
        let response = JsonResponse::success(json!({"name": "Test"}));
        let json_str = response.to_json_string();
        assert!(json_str.contains("\"success\": true"));
        assert!(json_str.contains("\"name\": \"Test\""));
    }

    #[test]
    fn test_result_to_json_success() {
        let result: Result<_, &str> = Ok(json!({"id": "123"}));
        let json_str = result_to_json(result);
        assert!(json_str.contains("\"success\": true"));
    }

    #[test]
    fn test_result_to_json_error() {
        let result: Result<(), _> = Err("Something went wrong");
        let json_str = result_to_json(result);
        assert!(json_str.contains("\"success\": false"));
        assert!(json_str.contains("Something went wrong"));
    }
}
