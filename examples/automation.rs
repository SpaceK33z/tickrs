//! AI Agent Automation Example
//!
//! This example demonstrates how to use tickrs for AI agent-driven task management.
//! It shows best practices for parsing JSON output, handling errors, and performing
//! common automation workflows.
//!
//! # Running the Example
//!
//! This example uses shell commands to demonstrate CLI automation patterns.
//! It requires tickrs to be installed and authenticated (`tickrs init`).
//!
//! ```bash
//! cargo run --example automation
//! ```

use std::process::Command;

/// Response structure for parsing tickrs JSON output
#[derive(Debug, serde::Deserialize)]
struct JsonResponse<T> {
    success: bool,
    data: Option<T>,
    error: Option<ErrorDetail>,
    message: Option<String>,
}

#[derive(Debug, serde::Deserialize)]
struct ErrorDetail {
    code: String,
    message: String,
}

#[derive(Debug, serde::Deserialize)]
struct ProjectListData {
    projects: Vec<Project>,
}

#[derive(Debug, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
struct Project {
    id: String,
    name: String,
    color: Option<String>,
}

#[derive(Debug, serde::Deserialize)]
struct TaskListData {
    tasks: Vec<Task>,
    count: usize,
}

#[derive(Debug, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
struct Task {
    id: String,
    project_id: String,
    title: String,
    status: i32,
    priority: i32,
}

#[derive(Debug, serde::Deserialize)]
struct TaskData {
    task: Task,
}

/// Run tickrs command and parse JSON output
fn run_tickrs(args: &[&str]) -> Result<String, String> {
    let output = Command::new("tickrs")
        .args(["--json"])
        .args(args)
        .output()
        .map_err(|e| format!("Failed to execute tickrs: {}", e))?;

    let stdout = String::from_utf8_lossy(&output.stdout).to_string();

    if !output.status.success() {
        // Try to parse error from JSON
        if let Ok(response) = serde_json::from_str::<JsonResponse<()>>(&stdout) {
            if let Some(err) = response.error {
                return Err(format!("[{}] {}", err.code, err.message));
            }
        }
        return Err(format!("Command failed: {}", stdout));
    }

    Ok(stdout)
}

/// List all projects and return the first one
fn get_first_project() -> Result<Project, String> {
    let output = run_tickrs(&["project", "list"])?;
    let response: JsonResponse<ProjectListData> =
        serde_json::from_str(&output).map_err(|e| format!("Failed to parse JSON: {}", e))?;

    if !response.success {
        return Err("API request failed".to_string());
    }

    response
        .data
        .and_then(|d| d.projects.into_iter().next())
        .ok_or_else(|| "No projects found".to_string())
}

/// List tasks in a project
fn list_tasks(project_id: &str) -> Result<Vec<Task>, String> {
    let output = run_tickrs(&["task", "list", "--project-id", project_id])?;
    let response: JsonResponse<TaskListData> =
        serde_json::from_str(&output).map_err(|e| format!("Failed to parse JSON: {}", e))?;

    if !response.success {
        return Err("API request failed".to_string());
    }

    Ok(response.data.map(|d| d.tasks).unwrap_or_default())
}

/// Create a new task and return its ID
fn create_task(project_id: &str, title: &str, priority: &str) -> Result<String, String> {
    let output = run_tickrs(&[
        "task",
        "create",
        "--project-id",
        project_id,
        "--title",
        title,
        "--priority",
        priority,
    ])?;

    let response: JsonResponse<TaskData> =
        serde_json::from_str(&output).map_err(|e| format!("Failed to parse JSON: {}", e))?;

    if !response.success {
        return Err("Failed to create task".to_string());
    }

    response
        .data
        .map(|d| d.task.id)
        .ok_or_else(|| "No task ID in response".to_string())
}

/// Complete a task
fn complete_task(project_id: &str, task_id: &str) -> Result<(), String> {
    let output = run_tickrs(&["task", "complete", task_id, "--project-id", project_id])?;
    let response: JsonResponse<()> =
        serde_json::from_str(&output).map_err(|e| format!("Failed to parse JSON: {}", e))?;

    if !response.success {
        return Err("Failed to complete task".to_string());
    }

    Ok(())
}

/// Delete a task
fn delete_task(project_id: &str, task_id: &str) -> Result<(), String> {
    let output = run_tickrs(&[
        "task",
        "delete",
        task_id,
        "--project-id",
        project_id,
        "--force",
    ])?;
    let response: JsonResponse<()> =
        serde_json::from_str(&output).map_err(|e| format!("Failed to parse JSON: {}", e))?;

    if !response.success {
        return Err("Failed to delete task".to_string());
    }

    Ok(())
}

fn main() {
    println!("=== tickrs AI Agent Automation Example ===\n");

    // Step 1: Get the first available project
    println!("1. Getting first project...");
    let project = match get_first_project() {
        Ok(p) => {
            println!("   Found project: {} ({})\n", p.name, p.id);
            p
        }
        Err(e) => {
            eprintln!("   Error: {}", e);
            eprintln!("   Make sure tickrs is installed and authenticated (tickrs init)");
            return;
        }
    };

    // Step 2: List existing tasks
    println!("2. Listing tasks in project...");
    match list_tasks(&project.id) {
        Ok(tasks) => {
            println!("   Found {} tasks:", tasks.len());
            for task in tasks.iter().take(5) {
                let status = if task.status == 2 { "[x]" } else { "[ ]" };
                println!("   {} {}", status, task.title);
            }
            if tasks.len() > 5 {
                println!("   ... and {} more", tasks.len() - 5);
            }
            println!();
        }
        Err(e) => {
            eprintln!("   Error listing tasks: {}", e);
        }
    }

    // Step 3: Create a test task
    println!("3. Creating a test task...");
    let task_id = match create_task(&project.id, "Test task from automation example", "medium") {
        Ok(id) => {
            println!("   Created task with ID: {}\n", id);
            id
        }
        Err(e) => {
            eprintln!("   Error creating task: {}", e);
            return;
        }
    };

    // Step 4: Complete the task
    println!("4. Completing the task...");
    match complete_task(&project.id, &task_id) {
        Ok(()) => println!("   Task marked as complete\n"),
        Err(e) => eprintln!("   Error completing task: {}", e),
    }

    // Step 5: Clean up - delete the test task
    println!("5. Cleaning up (deleting test task)...");
    match delete_task(&project.id, &task_id) {
        Ok(()) => println!("   Task deleted successfully\n"),
        Err(e) => eprintln!("   Error deleting task: {}", e),
    }

    println!("=== Automation example complete ===");
}
