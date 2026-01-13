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
            fs::create_dir_all(parent)
                .with_context(|| format!("Failed to create config directory: {}", parent.display()))?;
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
        let config_dir = dirs::config_dir()
            .with_context(|| "Could not determine config directory")?;
        Ok(config_dir.join("tickrs").join("config.toml"))
    }

    /// Get the data directory path (for token storage)
    pub fn data_dir() -> Result<PathBuf> {
        let data_dir = dirs::data_local_dir()
            .with_context(|| "Could not determine data directory")?;
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
            fs::create_dir_all(parent)
                .with_context(|| format!("Failed to create data directory: {}", parent.display()))?;
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

    fn with_temp_dirs<F>(test_fn: F)
    where
        F: FnOnce(),
    {
        // Create temp directories for testing
        let temp_dir = env::temp_dir().join(format!("tickrs_test_{}", std::process::id()));
        let config_dir = temp_dir.join("config");
        let data_dir = temp_dir.join("data");

        fs::create_dir_all(&config_dir).unwrap();
        fs::create_dir_all(&data_dir).unwrap();

        // Note: This test uses the actual dirs::config_dir() and dirs::data_local_dir()
        // For proper testing, you'd want to inject the paths
        test_fn();

        // Cleanup
        let _ = fs::remove_dir_all(&temp_dir);
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
}
