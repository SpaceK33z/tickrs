use clap::Subcommand;

#[derive(Subcommand, Debug)]
pub enum ProjectCommands {
    /// List all projects
    #[command(alias = "ls")]
    List,

    /// Show project details
    Show {
        /// Project ID
        id: String,
    },

    /// Set default project for commands
    Use {
        /// Project name or ID
        name_or_id: String,
    },

    /// Create a new project
    Create {
        /// Project name
        #[arg(long, short)]
        name: String,

        /// Project color (hex code, e.g., #FF5733)
        #[arg(long, short)]
        color: Option<String>,

        /// View mode (list, kanban, timeline)
        #[arg(long)]
        view_mode: Option<String>,

        /// Project kind (task, note)
        #[arg(long)]
        kind: Option<String>,
    },

    /// Update an existing project
    Update {
        /// Project ID
        id: String,

        /// New project name
        #[arg(long, short)]
        name: Option<String>,

        /// New project color (hex code)
        #[arg(long, short)]
        color: Option<String>,

        /// Archive/close the project
        #[arg(long)]
        closed: Option<bool>,
    },

    /// Delete a project
    Delete {
        /// Project ID
        id: String,

        /// Skip confirmation prompt
        #[arg(long)]
        force: bool,
    },
}
