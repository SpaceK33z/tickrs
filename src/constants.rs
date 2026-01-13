//! Application constants
//!
//! Centralized constants for API URLs, defaults, and configuration values.

/// TickTick API base URL
pub const API_BASE_URL: &str = "https://api.ticktick.com/open/v1";

/// OAuth authorization URL
pub const OAUTH_AUTH_URL: &str = "https://ticktick.com/oauth/authorize";

/// OAuth token exchange URL
pub const OAUTH_TOKEN_URL: &str = "https://ticktick.com/oauth/token";

/// OAuth redirect URI for local callback
pub const OAUTH_REDIRECT_URI: &str = "http://localhost:8080";

/// OAuth scopes required by the application
pub const OAUTH_SCOPES: &[&str] = &["tasks:write", "tasks:read"];

/// Special inbox project ID
pub const INBOX_PROJECT_ID: &str = "inbox";

/// Inbox project display name
pub const INBOX_PROJECT_NAME: &str = "Inbox";

/// Default color for new projects
pub const DEFAULT_PROJECT_COLOR: &str = "#FF1111";

/// Configuration file name
pub const CONFIG_FILE_NAME: &str = "config.toml";

/// Token file name
pub const TOKEN_FILE_NAME: &str = "token";

/// Application directory name (used in ~/.config/ and ~/.local/share/)
pub const APP_DIR_NAME: &str = "tickrs";

/// Environment variable for client ID
pub const ENV_CLIENT_ID: &str = "TICKTICK_CLIENT_ID";

/// Environment variable for client secret
pub const ENV_CLIENT_SECRET: &str = "TICKTICK_CLIENT_SECRET";

/// Environment variable for log level
pub const ENV_LOG_LEVEL: &str = "RUST_LOG";

/// Default log level
pub const DEFAULT_LOG_LEVEL: &str = "info";

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_api_base_url() {
        assert!(API_BASE_URL.starts_with("https://"));
        assert!(API_BASE_URL.contains("ticktick.com"));
    }

    #[test]
    fn test_oauth_urls() {
        assert!(OAUTH_AUTH_URL.starts_with("https://"));
        assert!(OAUTH_TOKEN_URL.starts_with("https://"));
        assert!(OAUTH_REDIRECT_URI.starts_with("http://localhost"));
    }

    #[test]
    fn test_oauth_scopes() {
        assert!(OAUTH_SCOPES.contains(&"tasks:write"));
        assert!(OAUTH_SCOPES.contains(&"tasks:read"));
    }

    #[test]
    fn test_inbox_constants() {
        assert_eq!(INBOX_PROJECT_ID, "inbox");
        assert_eq!(INBOX_PROJECT_NAME, "Inbox");
    }

    #[test]
    fn test_default_color_is_valid_hex() {
        assert!(DEFAULT_PROJECT_COLOR.starts_with('#'));
        assert_eq!(DEFAULT_PROJECT_COLOR.len(), 7);
    }

    #[test]
    fn test_env_var_names() {
        assert_eq!(ENV_CLIENT_ID, "TICKTICK_CLIENT_ID");
        assert_eq!(ENV_CLIENT_SECRET, "TICKTICK_CLIENT_SECRET");
    }
}
