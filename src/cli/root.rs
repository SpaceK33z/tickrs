use clap::{Parser, Subcommand};

use super::project::ProjectCommands;
use super::subtask::SubtaskCommands;
use super::task::TaskCommands;

/// TickTick CLI - AI agent-optimized task management
#[derive(Parser, Debug)]
#[command(name = "tickrs")]
#[command(author, version, about, long_about = None)]
pub struct Cli {
    /// Output in JSON format for machine consumption
    #[arg(long, global = true)]
    pub json: bool,

    /// Suppress all output (useful for scripts that only need exit codes)
    #[arg(long, short = 'q', global = true)]
    pub quiet: bool,

    /// Enable verbose output
    #[arg(long, short = 'v', global = true)]
    pub verbose: bool,

    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand, Debug)]
pub enum Commands {
    /// Initialize OAuth authentication with TickTick
    Init,

    /// Reset configuration and clear stored token
    Reset {
        /// Skip confirmation prompt
        #[arg(long)]
        force: bool,
    },

    /// Display version information
    Version,

    /// Project management commands
    #[command(subcommand)]
    Project(ProjectCommands),

    /// Task management commands
    #[command(subcommand)]
    Task(TaskCommands),

    /// Subtask management commands
    #[command(subcommand)]
    Subtask(SubtaskCommands),
}
