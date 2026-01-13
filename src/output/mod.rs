pub mod json;
pub mod text;

pub use json::{ErrorDetail, JsonResponse};

/// Output format for CLI commands
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum OutputFormat {
    /// Plain text output for humans
    #[default]
    Text,
    /// JSON output for machine consumption
    Json,
}

impl OutputFormat {
    /// Check if this is JSON format
    pub fn is_json(&self) -> bool {
        matches!(self, OutputFormat::Json)
    }
}
