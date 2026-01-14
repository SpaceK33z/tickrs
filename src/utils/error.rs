//! Error handling utilities
//!
//! Provides error codes, user-friendly error messages, and conversions
//! from various error types.

use serde::{Deserialize, Serialize};
use std::fmt;
use thiserror::Error;

/// Error codes for JSON output
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
#[allow(dead_code)] // Available for external use and JSON output
pub enum ErrorCode {
    /// Authentication required - user needs to run init
    AuthRequired,
    /// Token is invalid or expired
    AuthExpired,
    /// Requested resource was not found
    NotFound,
    /// Invalid request parameters
    InvalidRequest,
    /// Rate limited by API
    RateLimited,
    /// Server error from TickTick API
    ServerError,
    /// Network or connection error
    NetworkError,
    /// Failed to parse response
    ParseError,
    /// Configuration error
    ConfigError,
    /// Invalid date format
    InvalidDate,
    /// Project not specified and no default set
    NoProject,
    /// Unknown or unspecified error
    Unknown,
}

impl fmt::Display for ErrorCode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let code = match self {
            ErrorCode::AuthRequired => "AUTH_REQUIRED",
            ErrorCode::AuthExpired => "AUTH_EXPIRED",
            ErrorCode::NotFound => "NOT_FOUND",
            ErrorCode::InvalidRequest => "INVALID_REQUEST",
            ErrorCode::RateLimited => "RATE_LIMITED",
            ErrorCode::ServerError => "SERVER_ERROR",
            ErrorCode::NetworkError => "NETWORK_ERROR",
            ErrorCode::ParseError => "PARSE_ERROR",
            ErrorCode::ConfigError => "CONFIG_ERROR",
            ErrorCode::InvalidDate => "INVALID_DATE",
            ErrorCode::NoProject => "NO_PROJECT",
            ErrorCode::Unknown => "UNKNOWN",
        };
        write!(f, "{}", code)
    }
}

/// Application-level errors with user-friendly messages
#[derive(Debug, Error)]
#[allow(dead_code)] // Available for external use
pub enum AppError {
    #[error("Authentication required. Run 'tickrs init' to authenticate.")]
    AuthRequired,

    #[error("Your session has expired. Run 'tickrs init' to re-authenticate.")]
    AuthExpired,

    #[error("{0} not found. Verify the ID is correct.")]
    NotFound(String),

    #[error("Invalid request: {0}")]
    InvalidRequest(String),

    #[error("Rate limited by TickTick. Please wait a moment and try again.")]
    RateLimited,

    #[error("TickTick server error: {0}")]
    ServerError(String),

    #[error("Network error: {0}. Check your internet connection.")]
    NetworkError(String),

    #[error("Failed to parse API response: {0}")]
    ParseError(String),

    #[error("Configuration error: {0}")]
    ConfigError(String),

    #[error("Invalid date format: {0}. Try 'tomorrow', '2025-01-15', or 'in 3 days'.")]
    InvalidDate(String),

    #[error("No project specified. Use --project-id or run 'tickrs project use <name>' to set a default.")]
    NoProject,

    #[error("{0}")]
    Other(String),
}

#[allow(dead_code)] // Methods available for external use
impl AppError {
    /// Get the error code for this error
    pub fn code(&self) -> ErrorCode {
        match self {
            AppError::AuthRequired => ErrorCode::AuthRequired,
            AppError::AuthExpired => ErrorCode::AuthExpired,
            AppError::NotFound(_) => ErrorCode::NotFound,
            AppError::InvalidRequest(_) => ErrorCode::InvalidRequest,
            AppError::RateLimited => ErrorCode::RateLimited,
            AppError::ServerError(_) => ErrorCode::ServerError,
            AppError::NetworkError(_) => ErrorCode::NetworkError,
            AppError::ParseError(_) => ErrorCode::ParseError,
            AppError::ConfigError(_) => ErrorCode::ConfigError,
            AppError::InvalidDate(_) => ErrorCode::InvalidDate,
            AppError::NoProject => ErrorCode::NoProject,
            AppError::Other(_) => ErrorCode::Unknown,
        }
    }

    /// Get the error code as a string
    pub fn code_str(&self) -> &'static str {
        match self.code() {
            ErrorCode::AuthRequired => "AUTH_REQUIRED",
            ErrorCode::AuthExpired => "AUTH_EXPIRED",
            ErrorCode::NotFound => "NOT_FOUND",
            ErrorCode::InvalidRequest => "INVALID_REQUEST",
            ErrorCode::RateLimited => "RATE_LIMITED",
            ErrorCode::ServerError => "SERVER_ERROR",
            ErrorCode::NetworkError => "NETWORK_ERROR",
            ErrorCode::ParseError => "PARSE_ERROR",
            ErrorCode::ConfigError => "CONFIG_ERROR",
            ErrorCode::InvalidDate => "INVALID_DATE",
            ErrorCode::NoProject => "NO_PROJECT",
            ErrorCode::Unknown => "UNKNOWN",
        }
    }
}

/// Convert from API errors to application errors
impl From<crate::api::ApiError> for AppError {
    fn from(err: crate::api::ApiError) -> Self {
        match err {
            crate::api::ApiError::NotAuthenticated => AppError::AuthRequired,
            crate::api::ApiError::Unauthorized => AppError::AuthExpired,
            crate::api::ApiError::NotFound(resource) => AppError::NotFound(resource),
            crate::api::ApiError::BadRequest(msg) => AppError::InvalidRequest(msg),
            crate::api::ApiError::RateLimited => AppError::RateLimited,
            crate::api::ApiError::ServerError(msg) => AppError::ServerError(msg),
            crate::api::ApiError::NetworkError(e) => AppError::NetworkError(e.to_string()),
            crate::api::ApiError::ParseError(msg) => AppError::ParseError(msg),
        }
    }
}

/// Convert from date parse errors to application errors
impl From<crate::utils::date_parser::DateParseError> for AppError {
    fn from(err: crate::utils::date_parser::DateParseError) -> Self {
        match err {
            crate::utils::date_parser::DateParseError::InvalidFormat(s) => AppError::InvalidDate(s),
            crate::utils::date_parser::DateParseError::InvalidTimezone(tz) => {
                AppError::InvalidDate(format!("invalid timezone: {}", tz))
            }
            crate::utils::date_parser::DateParseError::PastDate(s) => {
                AppError::InvalidDate(format!("date is in the past: {}", s))
            }
        }
    }
}

/// Convert from anyhow errors
impl From<anyhow::Error> for AppError {
    fn from(err: anyhow::Error) -> Self {
        AppError::Other(err.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_codes() {
        assert_eq!(ErrorCode::AuthRequired.to_string(), "AUTH_REQUIRED");
        assert_eq!(ErrorCode::NotFound.to_string(), "NOT_FOUND");
        assert_eq!(ErrorCode::RateLimited.to_string(), "RATE_LIMITED");
    }

    #[test]
    fn test_error_code_serialization() {
        let code = ErrorCode::AuthRequired;
        let json = serde_json::to_string(&code).unwrap();
        assert_eq!(json, "\"AUTH_REQUIRED\"");

        let parsed: ErrorCode = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed, ErrorCode::AuthRequired);
    }

    #[test]
    fn test_app_error_messages() {
        let err = AppError::AuthRequired;
        assert!(err.to_string().contains("tickrs init"));

        let err = AppError::NotFound("Task".to_string());
        assert!(err.to_string().contains("Task"));
        assert!(err.to_string().contains("not found"));

        let err = AppError::NoProject;
        assert!(err.to_string().contains("--project-id"));
        assert!(err.to_string().contains("project use"));
    }

    #[test]
    fn test_app_error_codes() {
        assert_eq!(AppError::AuthRequired.code(), ErrorCode::AuthRequired);
        assert_eq!(AppError::AuthExpired.code(), ErrorCode::AuthExpired);
        assert_eq!(
            AppError::NotFound("test".to_string()).code(),
            ErrorCode::NotFound
        );
        assert_eq!(AppError::RateLimited.code(), ErrorCode::RateLimited);
        assert_eq!(AppError::NoProject.code(), ErrorCode::NoProject);
    }

    #[test]
    fn test_app_error_code_str() {
        assert_eq!(AppError::AuthRequired.code_str(), "AUTH_REQUIRED");
        assert_eq!(AppError::NoProject.code_str(), "NO_PROJECT");
        assert_eq!(
            AppError::InvalidDate("bad".to_string()).code_str(),
            "INVALID_DATE"
        );
    }

    #[test]
    fn test_from_api_error_not_authenticated() {
        let api_err = crate::api::ApiError::NotAuthenticated;
        let app_err: AppError = api_err.into();
        assert!(matches!(app_err, AppError::AuthRequired));
        assert_eq!(app_err.code(), ErrorCode::AuthRequired);
    }

    #[test]
    fn test_from_api_error_unauthorized() {
        let api_err = crate::api::ApiError::Unauthorized;
        let app_err: AppError = api_err.into();
        assert!(matches!(app_err, AppError::AuthExpired));
        assert_eq!(app_err.code(), ErrorCode::AuthExpired);
    }

    #[test]
    fn test_from_api_error_not_found() {
        let api_err = crate::api::ApiError::NotFound("Task".to_string());
        let app_err: AppError = api_err.into();
        match &app_err {
            AppError::NotFound(resource) => assert_eq!(resource, "Task"),
            _ => panic!("Expected NotFound variant"),
        }
        assert_eq!(app_err.code(), ErrorCode::NotFound);
    }

    #[test]
    fn test_from_api_error_bad_request() {
        let api_err = crate::api::ApiError::BadRequest("Invalid field".to_string());
        let app_err: AppError = api_err.into();
        match app_err {
            AppError::InvalidRequest(msg) => assert_eq!(msg, "Invalid field"),
            _ => panic!("Expected InvalidRequest variant"),
        }
    }

    #[test]
    fn test_from_api_error_rate_limited() {
        let api_err = crate::api::ApiError::RateLimited;
        let app_err: AppError = api_err.into();
        assert!(matches!(app_err, AppError::RateLimited));
        assert_eq!(app_err.code(), ErrorCode::RateLimited);
    }

    #[test]
    fn test_from_api_error_server_error() {
        let api_err = crate::api::ApiError::ServerError("500 Internal".to_string());
        let app_err: AppError = api_err.into();
        match app_err {
            AppError::ServerError(msg) => assert_eq!(msg, "500 Internal"),
            _ => panic!("Expected ServerError variant"),
        }
    }

    #[test]
    fn test_from_api_error_parse_error() {
        let api_err = crate::api::ApiError::ParseError("invalid JSON".to_string());
        let app_err: AppError = api_err.into();
        match app_err {
            AppError::ParseError(msg) => assert_eq!(msg, "invalid JSON"),
            _ => panic!("Expected ParseError variant"),
        }
    }

    #[test]
    fn test_from_date_parse_error_invalid_format() {
        let date_err =
            crate::utils::date_parser::DateParseError::InvalidFormat("not a date".to_string());
        let app_err: AppError = date_err.into();
        match &app_err {
            AppError::InvalidDate(msg) => assert_eq!(msg, "not a date"),
            _ => panic!("Expected InvalidDate variant"),
        }
        assert_eq!(app_err.code(), ErrorCode::InvalidDate);
    }

    #[test]
    fn test_from_date_parse_error_invalid_timezone() {
        let date_err =
            crate::utils::date_parser::DateParseError::InvalidTimezone("Bad/TZ".to_string());
        let app_err: AppError = date_err.into();
        match app_err {
            AppError::InvalidDate(msg) => assert!(msg.contains("Bad/TZ")),
            _ => panic!("Expected InvalidDate variant"),
        }
    }

    #[test]
    fn test_from_date_parse_error_past_date() {
        let date_err =
            crate::utils::date_parser::DateParseError::PastDate("2020-01-01".to_string());
        let app_err: AppError = date_err.into();
        match app_err {
            AppError::InvalidDate(msg) => {
                assert!(msg.contains("past"));
                assert!(msg.contains("2020-01-01"));
            }
            _ => panic!("Expected InvalidDate variant"),
        }
    }

    #[test]
    fn test_from_anyhow_error() {
        let anyhow_err = anyhow::anyhow!("Something went wrong");
        let app_err: AppError = anyhow_err.into();
        match &app_err {
            AppError::Other(msg) => assert!(msg.contains("Something went wrong")),
            _ => panic!("Expected Other variant"),
        }
        assert_eq!(app_err.code(), ErrorCode::Unknown);
    }

    #[test]
    fn test_all_error_codes_have_display() {
        // Verify all error codes can be displayed as SCREAMING_SNAKE_CASE
        let codes = vec![
            ErrorCode::AuthRequired,
            ErrorCode::AuthExpired,
            ErrorCode::NotFound,
            ErrorCode::InvalidRequest,
            ErrorCode::RateLimited,
            ErrorCode::ServerError,
            ErrorCode::NetworkError,
            ErrorCode::ParseError,
            ErrorCode::ConfigError,
            ErrorCode::InvalidDate,
            ErrorCode::NoProject,
            ErrorCode::Unknown,
        ];

        for code in codes {
            let display = code.to_string();
            assert!(display.chars().all(|c| c.is_uppercase() || c == '_'));
        }
    }
}
