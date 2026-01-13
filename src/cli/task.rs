use clap::Subcommand;

use crate::models::Priority;

#[derive(Subcommand, Debug)]
pub enum TaskCommands {
    /// List tasks in a project
    #[command(alias = "ls")]
    List {
        /// Project ID (uses default if not specified)
        #[arg(long, short)]
        project_id: Option<String>,

        /// Filter by priority
        #[arg(long)]
        priority: Option<Priority>,

        /// Filter by tag
        #[arg(long)]
        tag: Option<String>,

        /// Filter by status (complete/incomplete)
        #[arg(long)]
        status: Option<String>,
    },

    /// Show task details
    Show {
        /// Task ID
        id: String,

        /// Project ID (uses default if not specified)
        #[arg(long, short)]
        project_id: Option<String>,
    },

    /// Create a new task
    #[command(alias = "add")]
    Create {
        /// Task title
        #[arg(long, short)]
        title: String,

        /// Project ID (uses default if not specified)
        #[arg(long, short)]
        project_id: Option<String>,

        /// Task description/content
        #[arg(long, short)]
        content: Option<String>,

        /// Priority level
        #[arg(long)]
        priority: Option<Priority>,

        /// Tags (comma-separated)
        #[arg(long)]
        tags: Option<String>,

        /// Natural language date (sets both start and due date)
        #[arg(long)]
        date: Option<String>,

        /// Start date (ISO 8601 format)
        #[arg(long)]
        start: Option<String>,

        /// Due date (ISO 8601 format)
        #[arg(long)]
        due: Option<String>,

        /// Mark as all-day task
        #[arg(long)]
        all_day: bool,

        /// Timezone
        #[arg(long)]
        timezone: Option<String>,
    },

    /// Update an existing task
    Update {
        /// Task ID
        id: String,

        /// Project ID (uses default if not specified)
        #[arg(long, short)]
        project_id: Option<String>,

        /// New task title
        #[arg(long, short)]
        title: Option<String>,

        /// New task description/content
        #[arg(long, short)]
        content: Option<String>,

        /// New priority level
        #[arg(long)]
        priority: Option<Priority>,

        /// New tags (comma-separated)
        #[arg(long)]
        tags: Option<String>,

        /// Natural language date (sets both start and due date)
        #[arg(long)]
        date: Option<String>,

        /// New start date (ISO 8601 format)
        #[arg(long)]
        start: Option<String>,

        /// New due date (ISO 8601 format)
        #[arg(long)]
        due: Option<String>,

        /// Mark as all-day task
        #[arg(long)]
        all_day: Option<bool>,

        /// Timezone
        #[arg(long)]
        timezone: Option<String>,
    },

    /// Delete a task
    Delete {
        /// Task ID
        id: String,

        /// Project ID (uses default if not specified)
        #[arg(long, short)]
        project_id: Option<String>,

        /// Skip confirmation prompt
        #[arg(long)]
        force: bool,
    },

    /// Mark a task as complete
    Complete {
        /// Task ID
        id: String,

        /// Project ID (uses default if not specified)
        #[arg(long, short)]
        project_id: Option<String>,
    },

    /// Mark a task as incomplete
    Uncomplete {
        /// Task ID
        id: String,

        /// Project ID (uses default if not specified)
        #[arg(long, short)]
        project_id: Option<String>,
    },
}
