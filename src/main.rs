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

use api::{AuthHandler, CreateProjectRequest, TickTickClient, UpdateProjectRequest};
use cli::project::ProjectCommands;
use cli::{Cli, Commands};
use config::{Config, TokenStorage};
use constants::{ENV_CLIENT_ID, ENV_CLIENT_SECRET};
use output::json::{JsonResponse, ProjectData, ProjectListData, VersionData};
use output::text;
use output::OutputFormat;

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
        Commands::Task(_) => {
            // TODO: Implement in Phase 11
            anyhow::bail!("Task commands not yet implemented")
        }
        Commands::Subtask(_) => {
            // TODO: Implement in Phase 12
            anyhow::bail!("Subtask commands not yet implemented")
        }
    }
}

/// Initialize OAuth authentication
async fn cmd_init(format: OutputFormat, quiet: bool) -> anyhow::Result<()> {
    // Check if already initialized
    if TokenStorage::exists()? {
        let message = "Already authenticated. Use 'tickrs reset' to clear credentials and re-authenticate.";
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
            let response = JsonResponse::success_with_message(
                serde_json::json!({}),
                message,
            );
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
