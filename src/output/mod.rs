pub mod json;
pub mod text;

// Note: JsonResponse and ErrorDetail are re-exported for external consumers

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
    #[allow(dead_code)] // Available for external use
    pub fn is_json(&self) -> bool {
        matches!(self, OutputFormat::Json)
    }
}
