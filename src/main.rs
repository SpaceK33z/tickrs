mod api;
mod cli;
mod config;
mod constants;
mod models;
mod output;
mod utils;

use std::env;
use std::process::ExitCode;

use clap::Parser;

use api::{
    AuthHandler, CreateProjectRequest, CreateTaskRequest, TickTickClient, UpdateProjectRequest,
    UpdateTaskRequest,
};
use cli::project::ProjectCommands;
use cli::subtask::SubtaskCommands;
use cli::task::TaskCommands;
use cli::{Cli, Commands};
use config::{Config, TokenStorage};
use constants::{ENV_CLIENT_ID, ENV_CLIENT_SECRET};
use models::{Priority, Status};
use output::json::{
    JsonResponse, ProjectData, ProjectListData, SubtaskListData, TaskData, TaskListData,
    VersionData,
};
use output::text;
use output::OutputFormat;
use utils::date_parser::parse_date;

/// Application name
const APP_NAME: &str = env!("CARGO_PKG_NAME");
/// Application version
const APP_VERSION: &str = env!("CARGO_PKG_VERSION");

#[tokio::main]
async fn main() -> ExitCode {
    // Load environment variables from .env file if present
    let _ = dotenvy::dotenv();

    let cli = Cli::parse();

    // Determine output format
    let format = if cli.json {
        OutputFormat::Json
    } else {
        OutputFormat::Text
    };

    // Run the command and handle errors
    let result = run_command(cli.command, format, cli.quiet).await;

    match result {
        Ok(()) => ExitCode::SUCCESS,
        Err(e) => {
            if !cli.quiet {
                eprintln!("{}", e);
            }
            ExitCode::FAILURE
        }
    }
}

async fn run_command(command: Commands, format: OutputFormat, quiet: bool) -> anyhow::Result<()> {
    match command {
        Commands::Init => cmd_init(format, quiet).await,
        Commands::Reset { force } => cmd_reset(force, format, quiet),
        Commands::Version => cmd_version(format, quiet),
        Commands::Project(cmd) => cmd_project(cmd, format, quiet).await,
        Commands::Task(cmd) => cmd_task(cmd, format, quiet).await,
        Commands::Subtask(cmd) => cmd_subtask(cmd, format, quiet).await,
    }
}

/// Initialize OAuth authentication
async fn cmd_init(format: OutputFormat, quiet: bool) -> anyhow::Result<()> {
    // Check if already initialized
    if TokenStorage::exists()? {
        let message =
            "Already authenticated. Use 'tickrs reset' to clear credentials and re-authenticate.";
        if !quiet {
            output_message(format, message, "ALREADY_INITIALIZED")?;
        }
        return Ok(());
    }

    // Load client credentials from environment
    let client_id = env::var(ENV_CLIENT_ID).map_err(|_| {
        anyhow::anyhow!(
            "Missing {} environment variable. Set it to your TickTick OAuth client ID.",
            ENV_CLIENT_ID
        )
    })?;

    let client_secret = env::var(ENV_CLIENT_SECRET).map_err(|_| {
        anyhow::anyhow!(
            "Missing {} environment variable. Set it to your TickTick OAuth client secret.",
            ENV_CLIENT_SECRET
        )
    })?;

    if !quiet && format == OutputFormat::Text {
        println!("Opening browser for TickTick authorization...");
        println!("Please authorize the application in your browser.");
    }

    // Run OAuth flow
    let auth = AuthHandler::new(client_id, client_secret);
    let token = auth.run_oauth_flow().await?;

    // Save token
    TokenStorage::save(&token)?;

    // Initialize config
    let config = Config::default();
    config.save()?;

    let message = "Authentication successful";
    if !quiet {
        output_message(format, message, "SUCCESS")?;
    }

    Ok(())
}

/// Reset configuration and clear stored token
fn cmd_reset(force: bool, format: OutputFormat, quiet: bool) -> anyhow::Result<()> {
    // Check if anything exists to reset
    let token_exists = TokenStorage::exists()?;
    let config_path = Config::config_path()?;
    let config_exists = config_path.exists();

    if !token_exists && !config_exists {
        let message = "Nothing to reset - no configuration or token found";
        if !quiet {
            output_message(format, message, "NOTHING_TO_RESET")?;
        }
        return Ok(());
    }

    // Confirm unless --force is specified
    if !force && format == OutputFormat::Text {
        println!("This will delete your stored credentials and configuration.");
        println!("You will need to re-authenticate with 'tickrs init'.");
        print!("Continue? [y/N] ");
        std::io::Write::flush(&mut std::io::stdout())?;

        let mut input = String::new();
        std::io::stdin().read_line(&mut input)?;

        if !input.trim().eq_ignore_ascii_case("y") {
            println!("Aborted.");
            return Ok(());
        }
    }

    // Delete token and config
    if token_exists {
        TokenStorage::delete()?;
    }
    if config_exists {
        Config::delete()?;
    }

    let message = "Configuration and credentials cleared";
    if !quiet {
        output_message(format, message, "SUCCESS")?;
    }

    Ok(())
}

/// Display version information
fn cmd_version(format: OutputFormat, quiet: bool) -> anyhow::Result<()> {
    if quiet {
        return Ok(());
    }

    match format {
        OutputFormat::Json => {
            let data = VersionData {
                name: APP_NAME.to_string(),
                version: APP_VERSION.to_string(),
            };
            let response = JsonResponse::success(data);
            println!("{}", response.to_json_string());
        }
        OutputFormat::Text => {
            println!("{}", text::format_version(APP_NAME, APP_VERSION));
        }
    }

    Ok(())
}

/// Output a message in the appropriate format
fn output_message(format: OutputFormat, message: &str, code: &str) -> anyhow::Result<()> {
    match format {
        OutputFormat::Json => {
            let response = JsonResponse::success_with_message(serde_json::json!({}), message);
            println!("{}", response.to_json_string());
        }
        OutputFormat::Text => {
            if code == "SUCCESS" {
                println!("{}", text::format_success(message));
            } else {
                println!("{}", message);
            }
        }
    }
    Ok(())
}

/// Handle project commands
async fn cmd_project(
    cmd: ProjectCommands,
    format: OutputFormat,
    quiet: bool,
) -> anyhow::Result<()> {
    match cmd {
        ProjectCommands::List => cmd_project_list(format, quiet).await,
        ProjectCommands::Show { id } => cmd_project_show(&id, format, quiet).await,
        ProjectCommands::Use { name_or_id } => cmd_project_use(&name_or_id, format, quiet).await,
        ProjectCommands::Create {
            name,
            color,
            view_mode,
            kind,
        } => cmd_project_create(&name, color, view_mode, kind, format, quiet).await,
        ProjectCommands::Update {
            id,
            name,
            color,
            closed,
        } => cmd_project_update(&id, name, color, closed, format, quiet).await,
        ProjectCommands::Delete { id, force } => {
            cmd_project_delete(&id, force, format, quiet).await
        }
    }
}

/// List all projects
async fn cmd_project_list(format: OutputFormat, quiet: bool) -> anyhow::Result<()> {
    let client = TickTickClient::new()?;
    let projects = client.list_projects().await?;

    if quiet {
        return Ok(());
    }

    match format {
        OutputFormat::Json => {
            let data = ProjectListData { projects };
            let response = JsonResponse::success(data);
            println!("{}", response.to_json_string());
        }
        OutputFormat::Text => {
            println!("{}", text::format_project_list(&projects));
        }
    }

    Ok(())
}

/// Show project details
async fn cmd_project_show(id: &str, format: OutputFormat, quiet: bool) -> anyhow::Result<()> {
    let client = TickTickClient::new()?;
    let project = client.get_project(id).await?;

    if quiet {
        return Ok(());
    }

    match format {
        OutputFormat::Json => {
            let data = ProjectData { project };
            let response = JsonResponse::success(data);
            println!("{}", response.to_json_string());
        }
        OutputFormat::Text => {
            println!("{}", text::format_project_details(&project));
        }
    }

    Ok(())
}

/// Set default project for commands
async fn cmd_project_use(
    name_or_id: &str,
    format: OutputFormat,
    quiet: bool,
) -> anyhow::Result<()> {
    let client = TickTickClient::new()?;
    let projects = client.list_projects().await?;

    // Find project by name or ID
    let project = projects
        .iter()
        .find(|p| p.id == name_or_id || p.name.eq_ignore_ascii_case(name_or_id))
        .ok_or_else(|| anyhow::anyhow!("Project not found: {}", name_or_id))?;

    // Update config with the project ID
    let mut config = Config::load()?;
    config.default_project_id = Some(project.id.clone());
    config.save()?;

    if quiet {
        return Ok(());
    }

    let message = format!("Default project set to '{}'", project.name);
    match format {
        OutputFormat::Json => {
            let data = ProjectData {
                project: project.clone(),
            };
            let response = JsonResponse::success_with_message(data, &message);
            println!("{}", response.to_json_string());
        }
        OutputFormat::Text => {
            println!("{}", text::format_success(&message));
        }
    }

    Ok(())
}

/// Create a new project
async fn cmd_project_create(
    name: &str,
    color: Option<String>,
    view_mode: Option<String>,
    kind: Option<String>,
    format: OutputFormat,
    quiet: bool,
) -> anyhow::Result<()> {
    let client = TickTickClient::new()?;

    let request = CreateProjectRequest {
        name: name.to_string(),
        color,
        view_mode,
        kind,
    };

    let project = client.create_project(&request).await?;

    if quiet {
        return Ok(());
    }

    match format {
        OutputFormat::Json => {
            let data = ProjectData { project };
            let response = JsonResponse::success_with_message(data, "Project created successfully");
            println!("{}", response.to_json_string());
        }
        OutputFormat::Text => {
            println!(
                "{}",
                text::format_success_with_id("Project created", &project.id)
            );
        }
    }

    Ok(())
}

/// Update an existing project
async fn cmd_project_update(
    id: &str,
    name: Option<String>,
    color: Option<String>,
    closed: Option<bool>,
    format: OutputFormat,
    quiet: bool,
) -> anyhow::Result<()> {
    let client = TickTickClient::new()?;

    let request = UpdateProjectRequest {
        name,
        color,
        closed,
        view_mode: None,
    };

    let project = client.update_project(id, &request).await?;

    if quiet {
        return Ok(());
    }

    match format {
        OutputFormat::Json => {
            let data = ProjectData { project };
            let response = JsonResponse::success_with_message(data, "Project updated successfully");
            println!("{}", response.to_json_string());
        }
        OutputFormat::Text => {
            println!(
                "{}",
                text::format_success_with_id("Project updated", &project.id)
            );
        }
    }

    Ok(())
}

/// Delete a project
async fn cmd_project_delete(
    id: &str,
    force: bool,
    format: OutputFormat,
    quiet: bool,
) -> anyhow::Result<()> {
    // Confirm unless --force is specified
    if !force && format == OutputFormat::Text {
        print!("Delete project '{}'? [y/N] ", id);
        std::io::Write::flush(&mut std::io::stdout())?;

        let mut input = String::new();
        std::io::stdin().read_line(&mut input)?;

        if !input.trim().eq_ignore_ascii_case("y") {
            println!("Aborted.");
            return Ok(());
        }
    }

    let client = TickTickClient::new()?;
    client.delete_project(id).await?;

    if quiet {
        return Ok(());
    }

    let message = "Project deleted successfully";
    match format {
        OutputFormat::Json => {
            let response = JsonResponse::success_with_message(serde_json::json!({}), message);
            println!("{}", response.to_json_string());
        }
        OutputFormat::Text => {
            println!("{}", text::format_success(message));
        }
    }

    Ok(())
}

/// Handle task commands
async fn cmd_task(cmd: TaskCommands, format: OutputFormat, quiet: bool) -> anyhow::Result<()> {
    match cmd {
        TaskCommands::List {
            project_id,
            priority,
            tag,
            status,
        } => cmd_task_list(project_id, priority, tag, status, format, quiet).await,
        TaskCommands::Show { id, project_id } => {
            cmd_task_show(&id, project_id, format, quiet).await
        }
        TaskCommands::Create {
            title,
            project_id,
            content,
            priority,
            tags,
            date,
            start,
            due,
            all_day,
            timezone,
        } => {
            cmd_task_create(
                &title, project_id, content, priority, tags, date, start, due, all_day, timezone,
                format, quiet,
            )
            .await
        }
        TaskCommands::Update {
            id,
            project_id,
            title,
            content,
            priority,
            tags,
            date,
            start,
            due,
            all_day,
            timezone,
        } => {
            cmd_task_update(
                &id, project_id, title, content, priority, tags, date, start, due, all_day,
                timezone, format, quiet,
            )
            .await
        }
        TaskCommands::Delete {
            id,
            project_id,
            force,
        } => cmd_task_delete(&id, project_id, force, format, quiet).await,
        TaskCommands::Complete { id, project_id } => {
            cmd_task_complete(&id, project_id, format, quiet).await
        }
        TaskCommands::Uncomplete { id, project_id } => {
            cmd_task_uncomplete(&id, project_id, format, quiet).await
        }
    }
}

/// Get the project ID from argument or config default
fn get_project_id(project_id: Option<String>) -> anyhow::Result<String> {
    if let Some(id) = project_id {
        return Ok(id);
    }

    let config = Config::load()?;
    config.default_project_id.ok_or_else(|| {
        anyhow::anyhow!(
            "No project specified. Use --project-id or set a default with 'tickrs project use <name>'"
        )
    })
}

/// List tasks in a project
async fn cmd_task_list(
    project_id: Option<String>,
    priority_filter: Option<Priority>,
    tag_filter: Option<String>,
    status_filter: Option<String>,
    format: OutputFormat,
    quiet: bool,
) -> anyhow::Result<()> {
    let project_id = get_project_id(project_id)?;
    let client = TickTickClient::new()?;
    let mut tasks = client.list_tasks(&project_id).await?;

    // Apply filters
    if let Some(priority) = priority_filter {
        tasks.retain(|t| t.priority == priority);
    }

    if let Some(ref tag) = tag_filter {
        let tag_lower = tag.to_lowercase();
        tasks.retain(|t| t.tags.iter().any(|tt| tt.to_lowercase() == tag_lower));
    }

    if let Some(ref status) = status_filter {
        let status_lower = status.to_lowercase();
        match status_lower.as_str() {
            "complete" | "completed" | "done" => {
                tasks.retain(|t| t.status == Status::Complete);
            }
            "incomplete" | "pending" | "open" => {
                tasks.retain(|t| t.status == Status::Normal);
            }
            _ => {
                anyhow::bail!(
                    "Invalid status filter: {}. Use 'complete' or 'incomplete'",
                    status
                );
            }
        }
    }

    if quiet {
        return Ok(());
    }

    match format {
        OutputFormat::Json => {
            let count = tasks.len();
            let data = TaskListData { tasks, count };
            let response = JsonResponse::success(data);
            println!("{}", response.to_json_string());
        }
        OutputFormat::Text => {
            println!("{}", text::format_task_list(&tasks));
        }
    }

    Ok(())
}

/// Show task details
async fn cmd_task_show(
    task_id: &str,
    project_id: Option<String>,
    format: OutputFormat,
    quiet: bool,
) -> anyhow::Result<()> {
    let project_id = get_project_id(project_id)?;
    let client = TickTickClient::new()?;
    let task = client.get_task(&project_id, task_id).await?;

    if quiet {
        return Ok(());
    }

    match format {
        OutputFormat::Json => {
            let data = TaskData { task };
            let response = JsonResponse::success(data);
            println!("{}", response.to_json_string());
        }
        OutputFormat::Text => {
            println!("{}", text::format_task_details(&task));
        }
    }

    Ok(())
}

/// Create a new task
#[allow(clippy::too_many_arguments)]
async fn cmd_task_create(
    title: &str,
    project_id: Option<String>,
    content: Option<String>,
    priority: Option<Priority>,
    tags: Option<String>,
    date: Option<String>,
    start: Option<String>,
    due: Option<String>,
    all_day: bool,
    timezone: Option<String>,
    format: OutputFormat,
    quiet: bool,
) -> anyhow::Result<()> {
    let project_id = get_project_id(project_id)?;

    // Parse dates
    let (start_date, due_date) = parse_task_dates(date, start, due)?;

    // Parse tags
    let tags_vec = tags.map(|t| t.split(',').map(|s| s.trim().to_string()).collect());

    let request = CreateTaskRequest {
        title: title.to_string(),
        project_id: project_id.clone(),
        content,
        is_all_day: if all_day { Some(true) } else { None },
        start_date,
        due_date,
        priority: priority.map(|p| p.to_api_value()),
        time_zone: timezone,
        tags: tags_vec,
    };

    let client = TickTickClient::new()?;
    let task = client.create_task(&request).await?;

    if quiet {
        return Ok(());
    }

    match format {
        OutputFormat::Json => {
            let data = TaskData { task };
            let response = JsonResponse::success_with_message(data, "Task created successfully");
            println!("{}", response.to_json_string());
        }
        OutputFormat::Text => {
            println!("{}", text::format_success_with_id("Task created", &task.id));
        }
    }

    Ok(())
}

/// Update an existing task
#[allow(clippy::too_many_arguments)]
async fn cmd_task_update(
    task_id: &str,
    project_id: Option<String>,
    title: Option<String>,
    content: Option<String>,
    priority: Option<Priority>,
    tags: Option<String>,
    date: Option<String>,
    start: Option<String>,
    due: Option<String>,
    all_day: Option<bool>,
    timezone: Option<String>,
    format: OutputFormat,
    quiet: bool,
) -> anyhow::Result<()> {
    let project_id = get_project_id(project_id)?;

    // Parse dates
    let (start_date, due_date) = parse_task_dates(date, start, due)?;

    // Parse tags
    let tags_vec = tags.map(|t| t.split(',').map(|s| s.trim().to_string()).collect());

    let request = UpdateTaskRequest {
        id: task_id.to_string(),
        project_id: project_id.clone(),
        title,
        content,
        is_all_day: all_day,
        start_date,
        due_date,
        priority: priority.map(|p| p.to_api_value()),
        time_zone: timezone,
        tags: tags_vec,
        status: None,
    };

    let client = TickTickClient::new()?;
    let task = client.update_task(task_id, &request).await?;

    if quiet {
        return Ok(());
    }

    match format {
        OutputFormat::Json => {
            let data = TaskData { task };
            let response = JsonResponse::success_with_message(data, "Task updated successfully");
            println!("{}", response.to_json_string());
        }
        OutputFormat::Text => {
            println!("{}", text::format_success_with_id("Task updated", &task.id));
        }
    }

    Ok(())
}

/// Delete a task
async fn cmd_task_delete(
    task_id: &str,
    project_id: Option<String>,
    force: bool,
    format: OutputFormat,
    quiet: bool,
) -> anyhow::Result<()> {
    let project_id = get_project_id(project_id)?;

    // Confirm unless --force is specified
    if !force && format == OutputFormat::Text {
        print!("Delete task '{}'? [y/N] ", task_id);
        std::io::Write::flush(&mut std::io::stdout())?;

        let mut input = String::new();
        std::io::stdin().read_line(&mut input)?;

        if !input.trim().eq_ignore_ascii_case("y") {
            println!("Aborted.");
            return Ok(());
        }
    }

    let client = TickTickClient::new()?;
    client.delete_task(&project_id, task_id).await?;

    if quiet {
        return Ok(());
    }

    let message = "Task deleted successfully";
    match format {
        OutputFormat::Json => {
            let response = JsonResponse::success_with_message(serde_json::json!({}), message);
            println!("{}", response.to_json_string());
        }
        OutputFormat::Text => {
            println!("{}", text::format_success(message));
        }
    }

    Ok(())
}

/// Mark a task as complete
async fn cmd_task_complete(
    task_id: &str,
    project_id: Option<String>,
    format: OutputFormat,
    quiet: bool,
) -> anyhow::Result<()> {
    let project_id = get_project_id(project_id)?;

    let client = TickTickClient::new()?;
    client.complete_task(&project_id, task_id).await?;

    if quiet {
        return Ok(());
    }

    let message = "Task marked as complete";
    match format {
        OutputFormat::Json => {
            let response = JsonResponse::success_with_message(serde_json::json!({}), message);
            println!("{}", response.to_json_string());
        }
        OutputFormat::Text => {
            println!("{}", text::format_success(message));
        }
    }

    Ok(())
}

/// Mark a task as incomplete
async fn cmd_task_uncomplete(
    task_id: &str,
    project_id: Option<String>,
    format: OutputFormat,
    quiet: bool,
) -> anyhow::Result<()> {
    let project_id = get_project_id(project_id)?;

    let client = TickTickClient::new()?;
    let task = client.uncomplete_task(&project_id, task_id).await?;

    if quiet {
        return Ok(());
    }

    match format {
        OutputFormat::Json => {
            let data = TaskData { task };
            let response = JsonResponse::success_with_message(data, "Task marked as incomplete");
            println!("{}", response.to_json_string());
        }
        OutputFormat::Text => {
            println!("{}", text::format_success("Task marked as incomplete"));
        }
    }

    Ok(())
}

/// Parse task dates from various input formats
///
/// If `date` is provided, it sets both start and due date.
/// Otherwise, `start` and `due` can be specified separately.
fn parse_task_dates(
    date: Option<String>,
    start: Option<String>,
    due: Option<String>,
) -> anyhow::Result<(Option<String>, Option<String>)> {
    // If natural language date is provided, use it for both start and due
    if let Some(date_str) = date {
        let dt = parse_date(&date_str)?;
        let formatted = dt.format("%Y-%m-%dT%H:%M:%S%z").to_string();
        return Ok((Some(formatted.clone()), Some(formatted)));
    }

    // Parse individual dates
    let start_date = if let Some(start_str) = start {
        let dt = parse_date(&start_str)?;
        Some(dt.format("%Y-%m-%dT%H:%M:%S%z").to_string())
    } else {
        None
    };

    let due_date = if let Some(due_str) = due {
        let dt = parse_date(&due_str)?;
        Some(dt.format("%Y-%m-%dT%H:%M:%S%z").to_string())
    } else {
        None
    };

    Ok((start_date, due_date))
}

/// Handle subtask commands
async fn cmd_subtask(
    cmd: SubtaskCommands,
    format: OutputFormat,
    quiet: bool,
) -> anyhow::Result<()> {
    match cmd {
        SubtaskCommands::List {
            task_id,
            project_id,
        } => cmd_subtask_list(&task_id, project_id, format, quiet).await,
    }
}

/// List subtasks (checklist items) for a task
async fn cmd_subtask_list(
    task_id: &str,
    project_id: Option<String>,
    format: OutputFormat,
    quiet: bool,
) -> anyhow::Result<()> {
    let project_id = get_project_id(project_id)?;
    let client = TickTickClient::new()?;
    let task = client.get_task(&project_id, task_id).await?;

    let subtasks = task.items;

    if quiet {
        return Ok(());
    }

    match format {
        OutputFormat::Json => {
            let count = subtasks.len();
            let data = SubtaskListData { subtasks, count };
            let response = JsonResponse::success(data);
            println!("{}", response.to_json_string());
        }
        OutputFormat::Text => {
            println!("{}", text::format_subtask_list(&subtasks));
        }
    }

    Ok(())
}
