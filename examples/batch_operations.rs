//! Batch Operations Example
//!
//! This example demonstrates how to perform bulk task operations using tickrs.
//! It shows patterns for creating multiple tasks, processing task lists,
//! and handling batch completions.
//!
//! # Running the Example
//!
//! This example uses shell commands to demonstrate batch patterns.
//! It requires tickrs to be installed and authenticated (`tickrs init`).
//!
//! ```bash
//! cargo run --example batch_operations
//! ```

use serde::Deserialize;
use std::process::Command;

// ============================================================================
// Response Structures (same as json_parsing example)
// ============================================================================

#[derive(Debug, Deserialize)]
struct JsonResponse<T> {
    success: bool,
    data: Option<T>,
    error: Option<ErrorDetail>,
}

#[derive(Debug, Deserialize)]
struct ErrorDetail {
    code: String,
    message: String,
}

#[derive(Debug, Deserialize)]
struct ProjectListData {
    projects: Vec<Project>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct Project {
    id: String,
    name: String,
}

#[derive(Debug, Deserialize)]
struct TaskListData {
    tasks: Vec<Task>,
}

#[derive(Debug, Deserialize)]
struct TaskData {
    task: Task,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
#[allow(dead_code)]
struct Task {
    id: String,
    project_id: String,
    title: String,
    status: i32,
    priority: i32,
    #[serde(default)]
    tags: Vec<String>,
}

// ============================================================================
// Helper Functions
// ============================================================================

fn run_tickrs(args: &[&str]) -> Result<String, String> {
    let output = Command::new("tickrs")
        .args(["--json"])
        .args(args)
        .output()
        .map_err(|e| format!("Failed to execute tickrs: {}", e))?;

    let stdout = String::from_utf8_lossy(&output.stdout).to_string();

    if !output.status.success() {
        if let Ok(response) = serde_json::from_str::<JsonResponse<serde_json::Value>>(&stdout) {
            if let Some(err) = response.error {
                return Err(format!("[{}] {}", err.code, err.message));
            }
        }
        return Err(format!("Command failed: {}", stdout));
    }

    Ok(stdout)
}

fn parse_response<T: for<'de> Deserialize<'de>>(json: &str) -> Result<T, String> {
    // First parse to check success/error
    let value: serde_json::Value =
        serde_json::from_str(json).map_err(|e| format!("Parse error: {}", e))?;

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

// ============================================================================
// Batch Operation Functions
// ============================================================================

/// Batch create multiple tasks
fn batch_create_tasks(project_id: &str, tasks: &[(&str, &str)]) -> Vec<Result<String, String>> {
    tasks
        .iter()
        .map(|(title, priority)| {
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

            let data: TaskData = parse_response(&output)?;
            Ok(data.task.id)
        })
        .collect()
}

/// Batch complete multiple tasks
fn batch_complete_tasks(project_id: &str, task_ids: &[&str]) -> Vec<Result<(), String>> {
    task_ids
        .iter()
        .map(|task_id| {
            let output = run_tickrs(&["task", "complete", task_id, "--project-id", project_id])?;
            let _: serde_json::Value =
                serde_json::from_str(&output).map_err(|e| e.to_string())?;
            Ok(())
        })
        .collect()
}

/// Batch delete multiple tasks
fn batch_delete_tasks(project_id: &str, task_ids: &[&str]) -> Vec<Result<(), String>> {
    task_ids
        .iter()
        .map(|task_id| {
            let output = run_tickrs(&[
                "task",
                "delete",
                task_id,
                "--project-id",
                project_id,
                "--force",
            ])?;
            let _: serde_json::Value =
                serde_json::from_str(&output).map_err(|e| e.to_string())?;
            Ok(())
        })
        .collect()
}

/// Find tasks matching a filter predicate
fn find_tasks<F>(project_id: &str, filter: F) -> Result<Vec<Task>, String>
where
    F: Fn(&Task) -> bool,
{
    let output = run_tickrs(&["task", "list", "--project-id", project_id])?;
    let data: TaskListData = parse_response(&output)?;
    Ok(data.tasks.into_iter().filter(filter).collect())
}

/// Complete all tasks matching a filter
fn complete_matching_tasks<F>(project_id: &str, filter: F) -> Result<usize, String>
where
    F: Fn(&Task) -> bool,
{
    let matching = find_tasks(project_id, filter)?;
    let task_ids: Vec<&str> = matching.iter().map(|t| t.id.as_str()).collect();

    let results = batch_complete_tasks(project_id, &task_ids);
    let success_count = results.iter().filter(|r| r.is_ok()).count();

    Ok(success_count)
}

// ============================================================================
// Main Example
// ============================================================================

fn main() {
    println!("=== tickrs Batch Operations Example ===\n");

    // Get the first project
    println!("1. Getting project...");
    let project = match run_tickrs(&["project", "list"])
        .and_then(|json| parse_response::<ProjectListData>(&json))
        .map(|data| data.projects.into_iter().next())
    {
        Ok(Some(p)) => {
            println!("   Using project: {} ({})\n", p.name, p.id);
            p
        }
        Ok(None) => {
            eprintln!("   Error: No projects found");
            return;
        }
        Err(e) => {
            eprintln!("   Error: {}", e);
            eprintln!("   Make sure tickrs is installed and authenticated");
            return;
        }
    };

    // Batch create tasks
    println!("2. Batch creating tasks...");
    let tasks_to_create = [
        ("Batch task 1 - High priority", "high"),
        ("Batch task 2 - Medium priority", "medium"),
        ("Batch task 3 - Low priority", "low"),
        ("Batch task 4 - Medium priority", "medium"),
        ("Batch task 5 - High priority", "high"),
    ];

    let create_results = batch_create_tasks(&project.id, &tasks_to_create);
    let created_ids: Vec<String> = create_results
        .into_iter()
        .enumerate()
        .filter_map(|(i, result)| match result {
            Ok(id) => {
                println!("   Created: {} -> {}", tasks_to_create[i].0, id);
                Some(id)
            }
            Err(e) => {
                eprintln!("   Failed to create {}: {}", tasks_to_create[i].0, e);
                None
            }
        })
        .collect();
    println!("   Created {} tasks\n", created_ids.len());

    // Find tasks by priority
    println!("3. Finding high priority tasks...");
    match find_tasks(&project.id, |t| t.priority == 5) {
        Ok(tasks) => {
            println!("   Found {} high priority tasks:", tasks.len());
            for t in &tasks {
                println!("   - {}", t.title);
            }
        }
        Err(e) => eprintln!("   Error: {}", e),
    }
    println!();

    // Batch complete some tasks
    println!("4. Batch completing first 2 tasks...");
    if created_ids.len() >= 2 {
        let ids_to_complete: Vec<&str> = created_ids[..2].iter().map(|s| s.as_str()).collect();
        let complete_results = batch_complete_tasks(&project.id, &ids_to_complete);
        let completed = complete_results.iter().filter(|r| r.is_ok()).count();
        println!("   Completed {} tasks\n", completed);
    }

    // Complete all medium priority tasks using filter
    println!("5. Completing all medium priority tasks...");
    match complete_matching_tasks(&project.id, |t| {
        t.priority == 3 && t.title.starts_with("Batch task")
    }) {
        Ok(count) => println!("   Completed {} medium priority tasks\n", count),
        Err(e) => eprintln!("   Error: {}", e),
    }

    // List tasks with their status
    println!("6. Current task status:");
    match run_tickrs(&["task", "list", "--project-id", &project.id])
        .and_then(|json| parse_response::<TaskListData>(&json))
    {
        Ok(data) => {
            for t in data.tasks.iter().filter(|t| t.title.starts_with("Batch task")) {
                let status = if t.status == 2 { "[x]" } else { "[ ]" };
                println!("   {} {}", status, t.title);
            }
        }
        Err(e) => eprintln!("   Error: {}", e),
    }
    println!();

    // Clean up - batch delete all created tasks
    println!("7. Cleaning up (batch delete)...");
    let ids_to_delete: Vec<&str> = created_ids.iter().map(|s| s.as_str()).collect();
    let delete_results = batch_delete_tasks(&project.id, &ids_to_delete);
    let deleted = delete_results.iter().filter(|r| r.is_ok()).count();
    println!("   Deleted {} tasks\n", deleted);

    println!("=== Batch Operations Example Complete ===");
    println!("\nTip: For better performance with many tasks, consider using");
    println!("parallel execution or a job queue in production scenarios.");
}
