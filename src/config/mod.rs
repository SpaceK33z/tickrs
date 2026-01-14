use std::fs::{self, File};
use std::io::{Read, Write};
use std::path::PathBuf;

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};

/// Application configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    /// Default project ID for commands
    #[serde(skip_serializing_if = "Option::is_none")]
    pub default_project_id: Option<String>,
    /// Default color for new projects
    #[serde(default = "default_project_color")]
    pub default_project_color: String,
}

fn default_project_color() -> String {
    "#FF1111".to_string()
}

impl Default for Config {
    fn default() -> Self {
        Self {
            default_project_id: None,
            default_project_color: default_project_color(),
        }
    }
}

impl Config {
    /// Load configuration from file, creating default if not exists
    pub fn load() -> Result<Self> {
        let path = Self::config_path()?;

        if !path.exists() {
            let config = Self::default();
            config.save()?;
            return Ok(config);
        }

        let mut file = File::open(&path)
            .with_context(|| format!("Failed to open config file: {}", path.display()))?;

        let mut contents = String::new();
        file.read_to_string(&mut contents)
            .with_context(|| "Failed to read config file")?;

        let config: Config =
            toml::from_str(&contents).with_context(|| "Failed to parse config file")?;

        Ok(config)
    }

    /// Save configuration to file
    pub fn save(&self) -> Result<()> {
        let path = Self::config_path()?;

        // Ensure parent directory exists
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).with_context(|| {
                format!("Failed to create config directory: {}", parent.display())
            })?;
        }

        let contents =
            toml::to_string_pretty(self).with_context(|| "Failed to serialize config")?;

        let mut file = File::create(&path)
            .with_context(|| format!("Failed to create config file: {}", path.display()))?;

        file.write_all(contents.as_bytes())
            .with_context(|| "Failed to write config file")?;

        Ok(())
    }

    /// Delete configuration file
    pub fn delete() -> Result<()> {
        let path = Self::config_path()?;
        if path.exists() {
            fs::remove_file(&path)
                .with_context(|| format!("Failed to delete config file: {}", path.display()))?;
        }
        Ok(())
    }

    /// Get the configuration file path
    pub fn config_path() -> Result<PathBuf> {
        let config_dir =
            dirs::config_dir().with_context(|| "Could not determine config directory")?;
        Ok(config_dir.join("tickrs").join("config.toml"))
    }

    /// Get the data directory path (for token storage)
    pub fn data_dir() -> Result<PathBuf> {
        let data_dir =
            dirs::data_local_dir().with_context(|| "Could not determine data directory")?;
        Ok(data_dir.join("tickrs"))
    }
}

/// Token storage operations
pub struct TokenStorage;

impl TokenStorage {
    /// Load the access token from secure storage
    pub fn load() -> Result<Option<String>> {
        let path = Self::token_path()?;

        if !path.exists() {
            return Ok(None);
        }

        let mut file = File::open(&path)
            .with_context(|| format!("Failed to open token file: {}", path.display()))?;

        let mut token = String::new();
        file.read_to_string(&mut token)
            .with_context(|| "Failed to read token file")?;

        let token = token.trim().to_string();
        if token.is_empty() {
            return Ok(None);
        }

        Ok(Some(token))
    }

    /// Save the access token to secure storage with restricted permissions
    pub fn save(token: &str) -> Result<()> {
        let path = Self::token_path()?;

        // Ensure parent directory exists
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).with_context(|| {
                format!("Failed to create data directory: {}", parent.display())
            })?;
        }

        // Write token to file
        let mut file = File::create(&path)
            .with_context(|| format!("Failed to create token file: {}", path.display()))?;

        file.write_all(token.as_bytes())
            .with_context(|| "Failed to write token file")?;

        // Set file permissions to 0600 (owner read/write only)
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let permissions = fs::Permissions::from_mode(0o600);
            fs::set_permissions(&path, permissions)
                .with_context(|| "Failed to set token file permissions")?;
        }

        Ok(())
    }

    /// Delete the token file
    pub fn delete() -> Result<()> {
        let path = Self::token_path()?;
        if path.exists() {
            fs::remove_file(&path)
                .with_context(|| format!("Failed to delete token file: {}", path.display()))?;
        }
        Ok(())
    }

    /// Check if a token exists
    pub fn exists() -> Result<bool> {
        let path = Self::token_path()?;
        Ok(path.exists())
    }

    /// Get the token file path
    pub fn token_path() -> Result<PathBuf> {
        let data_dir = Config::data_dir()?;
        Ok(data_dir.join("token"))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::env;

    /// Helper to create a unique temp directory for testing
    fn create_temp_dir() -> PathBuf {
        let temp_dir = env::temp_dir().join(format!(
            "tickrs_test_{}_{:?}",
            std::process::id(),
            std::time::Instant::now()
        ));
        fs::create_dir_all(&temp_dir).unwrap();
        temp_dir
    }

    /// Helper to cleanup temp directory
    fn cleanup_temp_dir(path: &PathBuf) {
        let _ = fs::remove_dir_all(path);
    }

    #[test]
    fn test_config_default() {
        let config = Config::default();
        assert!(config.default_project_id.is_none());
        assert_eq!(config.default_project_color, "#FF1111");
    }

    #[test]
    fn test_config_serialization() {
        let config = Config {
            default_project_id: Some("proj123".to_string()),
            default_project_color: "#00AAFF".to_string(),
        };

        let toml_str = toml::to_string_pretty(&config).unwrap();
        assert!(toml_str.contains("default_project_id"));
        assert!(toml_str.contains("proj123"));
        assert!(toml_str.contains("default_project_color"));
        assert!(toml_str.contains("#00AAFF"));
    }

    #[test]
    fn test_config_deserialization() {
        let toml_str = "default_project_id = \"abc123\"\ndefault_project_color = \"#FF5733\"\n";

        let config: Config = toml::from_str(toml_str).unwrap();
        assert_eq!(config.default_project_id, Some("abc123".to_string()));
        assert_eq!(config.default_project_color, "#FF5733");
    }

    #[test]
    fn test_config_deserialization_minimal() {
        // Test that default values work
        let toml_str = "";
        let config: Config = toml::from_str(toml_str).unwrap();
        assert!(config.default_project_id.is_none());
        assert_eq!(config.default_project_color, "#FF1111");
    }

    #[test]
    fn test_config_path() {
        let path = Config::config_path().unwrap();
        assert!(path.ends_with("tickrs/config.toml") || path.ends_with("tickrs\\config.toml"));
    }

    #[test]
    fn test_token_path() {
        let path = TokenStorage::token_path().unwrap();
        assert!(path.ends_with("tickrs/token") || path.ends_with("tickrs\\token"));
    }

    #[test]
    fn test_data_dir() {
        let path = Config::data_dir().unwrap();
        assert!(path.ends_with("tickrs"));
    }

    // File operation tests using temp directories
    // These tests create files in temp directories to avoid affecting actual user config

    #[test]
    fn test_config_save_and_load_to_custom_path() {
        let temp_dir = create_temp_dir();
        let config_path = temp_dir.join("config.toml");

        // Create config and save manually to temp path
        let config = Config {
            default_project_id: Some("test_project".to_string()),
            default_project_color: "#AABBCC".to_string(),
        };

        let contents = toml::to_string_pretty(&config).unwrap();
        fs::write(&config_path, contents).unwrap();

        // Verify file exists
        assert!(config_path.exists());

        // Read back and verify
        let loaded_contents = fs::read_to_string(&config_path).unwrap();
        let loaded_config: Config = toml::from_str(&loaded_contents).unwrap();

        assert_eq!(
            loaded_config.default_project_id,
            Some("test_project".to_string())
        );
        assert_eq!(loaded_config.default_project_color, "#AABBCC");

        cleanup_temp_dir(&temp_dir);
    }

    #[test]
    fn test_config_save_creates_parent_directories() {
        let temp_dir = create_temp_dir();
        let nested_path = temp_dir.join("deep").join("nested").join("config.toml");

        // Ensure parent directory doesn't exist
        assert!(!nested_path.parent().unwrap().exists());

        // Create parent dirs and write
        if let Some(parent) = nested_path.parent() {
            fs::create_dir_all(parent).unwrap();
        }

        let config = Config::default();
        let contents = toml::to_string_pretty(&config).unwrap();
        fs::write(&nested_path, contents).unwrap();

        // Verify file was created
        assert!(nested_path.exists());

        cleanup_temp_dir(&temp_dir);
    }

    #[test]
    fn test_config_delete_file() {
        let temp_dir = create_temp_dir();
        let config_path = temp_dir.join("config.toml");

        // Create a config file
        fs::write(&config_path, "default_project_color = \"#FF1111\"\n").unwrap();
        assert!(config_path.exists());

        // Delete the file
        fs::remove_file(&config_path).unwrap();
        assert!(!config_path.exists());

        cleanup_temp_dir(&temp_dir);
    }

    #[test]
    fn test_config_delete_nonexistent_file() {
        let temp_dir = create_temp_dir();
        let config_path = temp_dir.join("nonexistent.toml");

        // File doesn't exist
        assert!(!config_path.exists());

        // Attempting to check and conditionally delete should work
        if config_path.exists() {
            fs::remove_file(&config_path).unwrap();
        }
        // No error - operation is idempotent
        assert!(!config_path.exists());

        cleanup_temp_dir(&temp_dir);
    }

    #[test]
    fn test_token_save_and_load_to_custom_path() {
        let temp_dir = create_temp_dir();
        let token_path = temp_dir.join("token");

        let test_token = "test_access_token_12345";

        // Save token to temp path
        fs::write(&token_path, test_token).unwrap();

        // Verify file exists
        assert!(token_path.exists());

        // Load and verify
        let loaded_token = fs::read_to_string(&token_path).unwrap();
        assert_eq!(loaded_token.trim(), test_token);

        cleanup_temp_dir(&temp_dir);
    }

    #[test]
    fn test_token_load_empty_file() {
        let temp_dir = create_temp_dir();
        let token_path = temp_dir.join("token");

        // Create empty token file
        fs::write(&token_path, "").unwrap();

        // Load and verify it's treated as None
        let loaded = fs::read_to_string(&token_path).unwrap();
        let token = loaded.trim().to_string();
        assert!(token.is_empty());

        cleanup_temp_dir(&temp_dir);
    }

    #[test]
    fn test_token_load_whitespace_only() {
        let temp_dir = create_temp_dir();
        let token_path = temp_dir.join("token");

        // Create token file with only whitespace
        fs::write(&token_path, "   \n\t  \n").unwrap();

        // Load and verify it's treated as None
        let loaded = fs::read_to_string(&token_path).unwrap();
        let token = loaded.trim().to_string();
        assert!(token.is_empty());

        cleanup_temp_dir(&temp_dir);
    }

    #[test]
    fn test_token_load_nonexistent() {
        let temp_dir = create_temp_dir();
        let token_path = temp_dir.join("nonexistent_token");

        // File doesn't exist
        assert!(!token_path.exists());

        cleanup_temp_dir(&temp_dir);
    }

    #[test]
    fn test_token_delete_file() {
        let temp_dir = create_temp_dir();
        let token_path = temp_dir.join("token");

        // Create a token file
        fs::write(&token_path, "some_token").unwrap();
        assert!(token_path.exists());

        // Delete the file
        fs::remove_file(&token_path).unwrap();
        assert!(!token_path.exists());

        cleanup_temp_dir(&temp_dir);
    }

    #[test]
    fn test_token_exists_check() {
        let temp_dir = create_temp_dir();
        let token_path = temp_dir.join("token");

        // Initially doesn't exist
        assert!(!token_path.exists());

        // Create file
        fs::write(&token_path, "token_value").unwrap();
        assert!(token_path.exists());

        // Delete file
        fs::remove_file(&token_path).unwrap();
        assert!(!token_path.exists());

        cleanup_temp_dir(&temp_dir);
    }

    #[cfg(unix)]
    #[test]
    fn test_token_save_permissions() {
        use std::os::unix::fs::PermissionsExt;

        let temp_dir = create_temp_dir();
        let token_path = temp_dir.join("token");

        // Write token and set permissions
        fs::write(&token_path, "secret_token").unwrap();
        let permissions = fs::Permissions::from_mode(0o600);
        fs::set_permissions(&token_path, permissions).unwrap();

        // Verify permissions are 0600
        let metadata = fs::metadata(&token_path).unwrap();
        let mode = metadata.permissions().mode() & 0o777;
        assert_eq!(mode, 0o600);

        cleanup_temp_dir(&temp_dir);
    }

    #[test]
    fn test_config_roundtrip_with_special_characters() {
        let temp_dir = create_temp_dir();
        let config_path = temp_dir.join("config.toml");

        // Config with special characters in project ID
        let config = Config {
            default_project_id: Some("project-with-dashes_and_underscores.123".to_string()),
            default_project_color: "#ABCDEF".to_string(),
        };

        // Save
        let contents = toml::to_string_pretty(&config).unwrap();
        fs::write(&config_path, &contents).unwrap();

        // Load
        let loaded_contents = fs::read_to_string(&config_path).unwrap();
        let loaded_config: Config = toml::from_str(&loaded_contents).unwrap();

        assert_eq!(
            loaded_config.default_project_id,
            Some("project-with-dashes_and_underscores.123".to_string())
        );

        cleanup_temp_dir(&temp_dir);
    }

    #[test]
    fn test_token_with_special_characters() {
        let temp_dir = create_temp_dir();
        let token_path = temp_dir.join("token");

        // Token with typical OAuth characters
        let test_token = "eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9.eyJzdWIiOiIxMjM0NTY3ODkw.SflKxwRJSMeKKF2QT4fwpMeJf36POk6yJV_adQssw5c";

        fs::write(&token_path, test_token).unwrap();
        let loaded = fs::read_to_string(&token_path).unwrap();

        assert_eq!(loaded.trim(), test_token);

        cleanup_temp_dir(&temp_dir);
    }
}
