use crate::models::{ChecklistItem, Priority, Project, Task};

/// Format a list of projects for text output
pub fn format_project_list(projects: &[Project]) -> String {
    if projects.is_empty() {
        return "No projects found.".to_string();
    }

    let mut output = String::from("Projects:\n");
    for project in projects {
        output.push_str(&format_project_line(project));
        output.push('\n');
    }
    output.push_str(&format!("\nTotal: {} project(s)", projects.len()));
    output
}

/// Format a single project line for list display
fn format_project_line(project: &Project) -> String {
    if project.is_inbox() {
        format!("- [{}] Inbox", project.id)
    } else if project.color.is_empty() {
        format!("- [{}] {}", project.id, project.name)
    } else {
        format!("- [{}] {} ({})", project.id, project.name, project.color)
    }
}

/// Format project details for show command
pub fn format_project_details(project: &Project) -> String {
    let mut output = String::new();
    output.push_str(&format!("Project: {}\n", project.id));
    output.push_str(&format!("Name: {}\n", project.name));
    if !project.color.is_empty() {
        output.push_str(&format!("Color: {}\n", project.color));
    }
    output.push_str(&format!("View Mode: {}\n", project.view_mode));
    output.push_str(&format!("Kind: {}\n", project.kind));
    output.push_str(&format!(
        "Closed: {}\n",
        if project.closed { "yes" } else { "no" }
    ));
    if let Some(ref group_id) = project.group_id {
        output.push_str(&format!("Group ID: {}\n", group_id));
    }
    output
}

/// Format a list of tasks for text output
pub fn format_task_list(tasks: &[Task]) -> String {
    if tasks.is_empty() {
        return "No tasks found.".to_string();
    }

    let mut output = String::from("Tasks:\n");
    for task in tasks {
        output.push_str(&format_task_line(task));
        output.push('\n');
    }
    output.push_str(&format!("\nTotal: {} task(s)", tasks.len()));
    output
}

/// Format a single task line for list display
fn format_task_line(task: &Task) -> String {
    let status_marker = if task.is_complete() { "[x]" } else { "[ ]" };
    let priority = format_priority_marker(&task.priority);
    let due = task
        .due_date
        .map(|d| format!(" (due: {})", d.format("%Y-%m-%d")))
        .unwrap_or_default();

    format!("{} {} {}{}", status_marker, priority, task.title, due)
}

/// Format priority as a visual marker
fn format_priority_marker(priority: &Priority) -> &'static str {
    match priority {
        Priority::None => "   ",
        Priority::Low => "[L]",
        Priority::Medium => "[M]",
        Priority::High => "[H]",
    }
}

/// Format task details for show command
pub fn format_task_details(task: &Task) -> String {
    let mut output = String::new();
    output.push_str(&format!("Task: {}\n", task.id));
    output.push_str(&format!("Title: {}\n", task.title));
    output.push_str(&format!("Project: {}\n", task.project_id));
    output.push_str(&format!("Status: {}\n", task.status));
    output.push_str(&format!("Priority: {}\n", task.priority));

    if let Some(ref due) = task.due_date {
        output.push_str(&format!("Due: {} UTC\n", due.format("%Y-%m-%d %H:%M:%S")));
    }
    if let Some(ref start) = task.start_date {
        output.push_str(&format!(
            "Start: {} UTC\n",
            start.format("%Y-%m-%d %H:%M:%S")
        ));
    }
    if task.is_all_day {
        output.push_str("All Day: yes\n");
    }
    if !task.content.is_empty() {
        output.push_str(&format!("Content: {}\n", task.content));
    }
    if !task.tags.is_empty() {
        output.push_str(&format!("Tags: {}\n", task.tags.join(", ")));
    }
    if !task.time_zone.is_empty() {
        output.push_str(&format!("Timezone: {}\n", task.time_zone));
    }

    // Show subtasks if present
    if !task.items.is_empty() {
        output.push_str(&format!("\nSubtasks ({}):\n", task.items.len()));
        for item in &task.items {
            output.push_str(&format_subtask_line(item));
            output.push('\n');
        }
    }

    output
}

/// Format a list of subtasks for text output
pub fn format_subtask_list(subtasks: &[ChecklistItem]) -> String {
    if subtasks.is_empty() {
        return "No subtasks found.".to_string();
    }

    let mut output = String::from("Subtasks:\n");
    for subtask in subtasks {
        output.push_str(&format_subtask_line(subtask));
        output.push('\n');
    }
    output.push_str(&format!("\nTotal: {} subtask(s)", subtasks.len()));
    output
}

/// Format a single subtask line for list display
fn format_subtask_line(subtask: &ChecklistItem) -> String {
    let status_marker = if subtask.is_complete() { "[x]" } else { "[ ]" };
    format!("  {} {}", status_marker, subtask.title)
}

/// Format a success message
pub fn format_success(message: &str) -> String {
    format!("OK: {}", message)
}

/// Format a success message with an ID
pub fn format_success_with_id(message: &str, id: &str) -> String {
    format!("OK: {}\n  ID: {}", message, id)
}

/// Format an error message
#[allow(dead_code)] // Available for external use
pub fn format_error(message: &str) -> String {
    format!("Error: {}", message)
}

/// Format version information
pub fn format_version(name: &str, version: &str) -> String {
    format!("{} {}", name, version)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::{Priority, Status};
    use chrono::Utc;

    fn sample_project() -> Project {
        Project {
            id: "proj123".to_string(),
            name: "Work".to_string(),
            color: "#FF5733".to_string(),
            sort_order: 0,
            closed: false,
            group_id: None,
            view_mode: "list".to_string(),
            permission: None,
            kind: "TASK".to_string(),
        }
    }

    fn sample_task() -> Task {
        Task {
            id: "task123".to_string(),
            project_id: "proj123".to_string(),
            title: "Complete report".to_string(),
            is_all_day: false,
            completed_time: None,
            content: "Finish the quarterly report".to_string(),
            due_date: None,
            items: vec![],
            priority: Priority::Medium,
            reminders: vec![],
            repeat_flag: None,
            sort_order: 0,
            start_date: None,
            status: Status::Normal,
            time_zone: "UTC".to_string(),
            tags: vec!["work".to_string(), "urgent".to_string()],
        }
    }

    #[test]
    fn test_format_project_list() {
        let projects = vec![sample_project(), Project::inbox()];
        let output = format_project_list(&projects);
        assert!(output.contains("Projects:"));
        assert!(output.contains("[proj123] Work (#FF5733)"));
        assert!(output.contains("[inbox] Inbox"));
        assert!(output.contains("Total: 2 project(s)"));
    }

    #[test]
    fn test_format_empty_project_list() {
        let output = format_project_list(&[]);
        assert_eq!(output, "No projects found.");
    }

    #[test]
    fn test_format_project_details() {
        let project = sample_project();
        let output = format_project_details(&project);
        assert!(output.contains("Project: proj123"));
        assert!(output.contains("Name: Work"));
        assert!(output.contains("Color: #FF5733"));
        assert!(output.contains("View Mode: list"));
    }

    #[test]
    fn test_format_task_list() {
        let mut task = sample_task();
        task.due_date = Some(Utc::now());
        let tasks = vec![task];
        let output = format_task_list(&tasks);
        assert!(output.contains("Tasks:"));
        assert!(output.contains("[ ] [M] Complete report"));
        assert!(output.contains("Total: 1 task(s)"));
    }

    #[test]
    fn test_format_empty_task_list() {
        let output = format_task_list(&[]);
        assert_eq!(output, "No tasks found.");
    }

    #[test]
    fn test_format_task_details() {
        let task = sample_task();
        let output = format_task_details(&task);
        assert!(output.contains("Task: task123"));
        assert!(output.contains("Title: Complete report"));
        assert!(output.contains("Status: incomplete"));
        assert!(output.contains("Priority: medium"));
        assert!(output.contains("Tags: work, urgent"));
    }

    #[test]
    fn test_format_success() {
        let output = format_success("Task created successfully");
        assert_eq!(output, "OK: Task created successfully");
    }

    #[test]
    fn test_format_success_with_id() {
        let output = format_success_with_id("Task created", "task123");
        assert!(output.contains("OK: Task created"));
        assert!(output.contains("ID: task123"));
    }

    #[test]
    fn test_format_error() {
        let output = format_error("Task not found");
        assert_eq!(output, "Error: Task not found");
    }

    #[test]
    fn test_format_version() {
        let output = format_version("tickrs", "0.1.0");
        assert_eq!(output, "tickrs 0.1.0");
    }

    #[test]
    fn test_format_priority_markers() {
        assert_eq!(format_priority_marker(&Priority::None), "   ");
        assert_eq!(format_priority_marker(&Priority::Low), "[L]");
        assert_eq!(format_priority_marker(&Priority::Medium), "[M]");
        assert_eq!(format_priority_marker(&Priority::High), "[H]");
    }

    #[test]
    fn test_format_subtask_list() {
        let subtasks = vec![
            ChecklistItem {
                id: "sub1".to_string(),
                title: "Step 1".to_string(),
                status: 0,
                completed_time: 0,
                is_all_day: false,
                sort_order: 0,
                start_date: None,
                time_zone: "UTC".to_string(),
            },
            ChecklistItem {
                id: "sub2".to_string(),
                title: "Step 2".to_string(),
                status: 1,
                completed_time: 0,
                is_all_day: false,
                sort_order: 1,
                start_date: None,
                time_zone: "UTC".to_string(),
            },
        ];
        let output = format_subtask_list(&subtasks);
        assert!(output.contains("Subtasks:"));
        assert!(output.contains("[ ] Step 1"));
        assert!(output.contains("[x] Step 2"));
        assert!(output.contains("Total: 2 subtask(s)"));
    }
}
