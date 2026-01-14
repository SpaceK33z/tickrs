//! JSON Output Parsing Example
//!
//! This example demonstrates how to parse JSON output from tickrs commands.
//! It shows the structure of success and error responses, and best practices
//! for handling different response types.
//!
//! # Running the Example
//!
//! ```bash
//! cargo run --example json_parsing
//! ```

use serde::Deserialize;

// ============================================================================
// Response Structures
// ============================================================================

/// Standard JSON response wrapper from tickrs
///
/// All tickrs commands with `--json` flag return this structure.
#[derive(Debug, Deserialize)]
pub struct JsonResponse<T> {
    /// Whether the operation succeeded
    pub success: bool,
    /// Data payload (present on success)
    pub data: Option<T>,
    /// Error details (present on failure)
    pub error: Option<ErrorDetail>,
    /// Human-readable message
    #[serde(default)]
    pub message: Option<String>,
}

/// Error details for failed operations
#[derive(Debug, Deserialize)]
pub struct ErrorDetail {
    /// Error code (e.g., "AUTH_REQUIRED", "NOT_FOUND")
    pub code: String,
    /// Human-readable error message
    pub message: String,
    /// Additional error context
    #[serde(default)]
    pub details: Option<serde_json::Value>,
}

// ============================================================================
// Data Structures
// ============================================================================

/// Project list response data
#[derive(Debug, Deserialize)]
pub struct ProjectListData {
    pub projects: Vec<Project>,
}

/// Single project response data
#[allow(dead_code)]
#[derive(Debug, Deserialize)]
pub struct ProjectData {
    pub project: Project,
}

/// Task list response data
#[derive(Debug, Deserialize)]
pub struct TaskListData {
    pub tasks: Vec<Task>,
    pub count: usize,
}

/// Single task response data
#[derive(Debug, Deserialize)]
pub struct TaskData {
    pub task: Task,
}

/// Subtask list response data
#[allow(dead_code)]
#[derive(Debug, Deserialize)]
pub struct SubtaskListData {
    pub subtasks: Vec<Subtask>,
    pub count: usize,
}

/// Version response data
#[derive(Debug, Deserialize)]
pub struct VersionData {
    pub name: String,
    pub version: String,
}

// ============================================================================
// Domain Models
// ============================================================================

/// Project model
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Project {
    pub id: String,
    pub name: String,
    #[serde(default)]
    pub color: Option<String>,
    #[serde(default)]
    pub sort_order: i64,
    #[serde(default)]
    pub closed: bool,
    #[serde(default)]
    pub group_id: Option<String>,
    #[serde(default)]
    pub view_mode: Option<String>,
    #[serde(default)]
    pub kind: Option<String>,
}

/// Task model
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Task {
    pub id: String,
    pub project_id: String,
    pub title: String,
    #[serde(default)]
    pub status: i32,
    #[serde(default)]
    pub priority: i32,
    #[serde(default)]
    pub content: Option<String>,
    #[serde(default)]
    pub due_date: Option<String>,
    #[serde(default)]
    pub start_date: Option<String>,
    #[serde(default)]
    pub tags: Vec<String>,
    #[serde(default)]
    pub items: Vec<Subtask>,
}

/// Subtask (checklist item) model
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Subtask {
    pub id: String,
    pub title: String,
    #[serde(default)]
    pub status: i32,
}

// ============================================================================
// Parsing Examples
// ============================================================================

fn main() {
    println!("=== tickrs JSON Parsing Examples ===\n");

    // Example 1: Parse project list response
    println!("1. Parsing project list response:");
    // Note: Using r##"..."## to allow # characters in color codes
    let project_list_json = r##"{
        "success": true,
        "data": {
            "projects": [
                {"id": "inbox", "name": "Inbox", "color": null, "sortOrder": 0, "closed": false},
                {"id": "proj123", "name": "Work", "color": "#FF5733", "sortOrder": 1, "closed": false}
            ]
        }
    }"##;

    match serde_json::from_str::<JsonResponse<ProjectListData>>(project_list_json) {
        Ok(response) => {
            if response.success {
                if let Some(data) = response.data {
                    println!("   Found {} projects:", data.projects.len());
                    for p in &data.projects {
                        println!(
                            "   - {} ({}): color={:?}",
                            p.name,
                            p.id,
                            p.color.as_deref().unwrap_or("none")
                        );
                    }
                }
            }
        }
        Err(e) => println!("   Parse error: {}", e),
    }
    println!();

    // Example 2: Parse task list response
    println!("2. Parsing task list response:");
    let task_list_json = r##"{
        "success": true,
        "data": {
            "tasks": [
                {
                    "id": "task1",
                    "projectId": "proj123",
                    "title": "Complete report",
                    "status": 0,
                    "priority": 3,
                    "dueDate": "2026-01-15T14:00:00Z",
                    "tags": ["work", "urgent"]
                },
                {
                    "id": "task2",
                    "projectId": "proj123",
                    "title": "Review PR",
                    "status": 2,
                    "priority": 1
                }
            ],
            "count": 2
        }
    }"##;

    match serde_json::from_str::<JsonResponse<TaskListData>>(task_list_json) {
        Ok(response) => {
            if response.success {
                if let Some(data) = response.data {
                    println!("   Found {} tasks:", data.count);
                    for t in &data.tasks {
                        let status = if t.status == 2 { "complete" } else { "pending" };
                        let priority = match t.priority {
                            5 => "high",
                            3 => "medium",
                            1 => "low",
                            _ => "none",
                        };
                        println!("   - [{}] {} (priority: {})", status, t.title, priority);
                        if !t.tags.is_empty() {
                            println!("     tags: {}", t.tags.join(", "));
                        }
                    }
                }
            }
        }
        Err(e) => println!("   Parse error: {}", e),
    }
    println!();

    // Example 3: Parse error response
    println!("3. Parsing error response:");
    let error_json = r##"{
        "success": false,
        "error": {
            "code": "AUTH_REQUIRED",
            "message": "Authentication required. Run 'tickrs init' to authenticate."
        }
    }"##;

    match serde_json::from_str::<JsonResponse<serde_json::Value>>(error_json) {
        Ok(response) => {
            if !response.success {
                if let Some(err) = response.error {
                    println!("   Error code: {}", err.code);
                    println!("   Message: {}", err.message);

                    // Handle specific error codes
                    match err.code.as_str() {
                        "AUTH_REQUIRED" => println!("   Action: Run 'tickrs init' to authenticate"),
                        "AUTH_EXPIRED" => {
                            println!("   Action: Run 'tickrs init' to re-authenticate")
                        }
                        "NOT_FOUND" => println!("   Action: Check the resource ID"),
                        "RATE_LIMITED" => println!("   Action: Wait and retry"),
                        _ => println!("   Action: Check the error message"),
                    }
                }
            }
        }
        Err(e) => println!("   Parse error: {}", e),
    }
    println!();

    // Example 4: Parse task with subtasks
    println!("4. Parsing task with subtasks:");
    let task_json = r##"{
        "success": true,
        "data": {
            "task": {
                "id": "task123",
                "projectId": "proj456",
                "title": "Project setup",
                "status": 0,
                "priority": 5,
                "content": "Set up the new project repository",
                "items": [
                    {"id": "sub1", "title": "Create repository", "status": 2},
                    {"id": "sub2", "title": "Add CI/CD", "status": 0},
                    {"id": "sub3", "title": "Write README", "status": 0}
                ]
            }
        },
        "message": "Task retrieved successfully"
    }"##;

    match serde_json::from_str::<JsonResponse<TaskData>>(task_json) {
        Ok(response) => {
            if response.success {
                if let Some(data) = response.data {
                    let task = &data.task;
                    println!("   Task: {}", task.title);
                    if let Some(content) = &task.content {
                        println!("   Description: {}", content);
                    }
                    println!("   Subtasks:");
                    for sub in &task.items {
                        let check = if sub.status == 2 { "[x]" } else { "[ ]" };
                        println!("   {} {}", check, sub.title);
                    }
                }
            }
        }
        Err(e) => println!("   Parse error: {}", e),
    }
    println!();

    // Example 5: Parse version response
    println!("5. Parsing version response:");
    let version_json = r##"{
        "success": true,
        "data": {
            "name": "tickrs",
            "version": "0.1.0"
        }
    }"##;

    match serde_json::from_str::<JsonResponse<VersionData>>(version_json) {
        Ok(response) => {
            if response.success {
                if let Some(data) = response.data {
                    println!("   {} v{}", data.name, data.version);
                }
            }
        }
        Err(e) => println!("   Parse error: {}", e),
    }
    println!();

    // Example 6: Generic response handling helper
    println!("6. Using generic response handler:");
    demonstrate_generic_handler();

    println!("\n=== JSON Parsing Examples Complete ===");
}

/// Demonstrates a generic response handling pattern
fn demonstrate_generic_handler() {
    /// Generic function to handle tickrs JSON responses
    fn handle_response<T: for<'de> Deserialize<'de>>(json: &str) -> Result<T, String> {
        // First parse as a generic Value to check success/error
        let value: serde_json::Value =
            serde_json::from_str(json).map_err(|e| format!("JSON parse error: {}", e))?;

        let success = value
            .get("success")
            .and_then(|v| v.as_bool())
            .unwrap_or(false);

        if !success {
            if let Some(err) = value.get("error") {
                let code = err
                    .get("code")
                    .and_then(|v| v.as_str())
                    .unwrap_or("UNKNOWN");
                let message = err
                    .get("message")
                    .and_then(|v| v.as_str())
                    .unwrap_or("Unknown error");
                return Err(format!("[{}] {}", code, message));
            }
            return Err("Request failed".to_string());
        }

        // Parse the data field
        let data = value
            .get("data")
            .ok_or_else(|| "No data in response".to_string())?;

        serde_json::from_value(data.clone()).map_err(|e| format!("Data parse error: {}", e))
    }

    // Use the generic handler
    let json = r##"{"success": true, "data": {"name": "tickrs", "version": "0.1.0"}}"##;
    match handle_response::<VersionData>(json) {
        Ok(data) => println!("   Success: {} v{}", data.name, data.version),
        Err(e) => println!("   Error: {}", e),
    }
}
