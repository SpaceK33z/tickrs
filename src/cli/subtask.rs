use clap::Subcommand;

#[derive(Subcommand, Debug)]
pub enum SubtaskCommands {
    /// List subtasks (checklist items) for a task
    #[command(alias = "ls")]
    List {
        /// Task ID
        task_id: String,

        /// Project ID (uses default if not specified)
        #[arg(long, short)]
        project_id: Option<String>,

        /// Project name (alternative to project_id)
        #[arg(long, short = 'n')]
        project_name: Option<String>,
    },
}
