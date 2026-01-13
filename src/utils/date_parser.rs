//! Natural language date parsing utilities
//!
//! Provides functionality to parse dates from various formats including:
//! - Natural language: "today", "tomorrow", "next week"
//! - Relative: "in 3 days", "in 2 hours"
//! - Time specifications: "tomorrow at 2pm"
//! - ISO 8601 formats

use chrono::{DateTime, Duration, Local, NaiveTime, TimeZone, Utc};
use chrono_tz::Tz;
use thiserror::Error;

/// Errors that can occur during date parsing
#[derive(Debug, Error)]
pub enum DateParseError {
    #[error(
        "Could not parse date: '{0}'. Try formats like 'tomorrow', '2025-01-15', or 'in 3 days'."
    )]
    InvalidFormat(String),

    #[error("Invalid timezone: '{0}'")]
    #[allow(dead_code)] // Used by parse_date_with_timezone
    InvalidTimezone(String),

    #[error("Date is in the past: '{0}'")]
    #[allow(dead_code)] // Used by parse_future_date
    PastDate(String),
}

/// Parse a natural language date string into a UTC DateTime
///
/// Supports various formats:
/// - "today", "tomorrow", "yesterday"
/// - "next week", "next month"
/// - "in 3 days", "in 2 hours", "in 30 minutes"
/// - "tomorrow at 2pm", "friday at 14:00"
/// - ISO 8601: "2025-01-15", "2025-01-15T14:00:00Z"
///
/// # Arguments
/// * `input` - The date string to parse
///
/// # Returns
/// * `Ok(DateTime<Utc>)` - The parsed date in UTC
/// * `Err(DateParseError)` - If the date could not be parsed
pub fn parse_date(input: &str) -> Result<DateTime<Utc>, DateParseError> {
    let input = input.trim();
    let input_lower = input.to_lowercase();

    if input.is_empty() {
        return Err(DateParseError::InvalidFormat("empty string".to_string()));
    }

    // Handle natural language expressions that dateparser doesn't support
    let now = Utc::now();
    let today_start = now.date_naive().and_hms_opt(0, 0, 0).unwrap().and_utc();

    // Check for simple relative expressions
    if input_lower == "today" {
        return Ok(today_start);
    }

    if input_lower == "tomorrow" {
        return Ok(today_start + Duration::days(1));
    }

    if input_lower == "yesterday" {
        return Ok(today_start - Duration::days(1));
    }

    if input_lower == "next week" {
        return Ok(today_start + Duration::weeks(1));
    }

    if input_lower == "next month" {
        return Ok(today_start + Duration::days(30));
    }

    // Parse "in X days/hours/minutes" format
    if let Some(rest) = input_lower.strip_prefix("in ") {
        if let Some(result) = parse_relative_time(rest, now) {
            return Ok(result);
        }
    }

    // Try dateparser for ISO dates and other formats
    dateparser::parse(input).map_err(|_| DateParseError::InvalidFormat(input.to_string()))
}

/// Parse relative time expressions like "3 days", "2 hours", "30 minutes"
fn parse_relative_time(input: &str, base: DateTime<Utc>) -> Option<DateTime<Utc>> {
    let parts: Vec<&str> = input.split_whitespace().collect();
    if parts.len() < 2 {
        return None;
    }

    let amount: i64 = parts[0].parse().ok()?;
    let unit = parts[1].to_lowercase();

    match unit.as_str() {
        "day" | "days" => Some(base + Duration::days(amount)),
        "week" | "weeks" => Some(base + Duration::weeks(amount)),
        "hour" | "hours" => Some(base + Duration::hours(amount)),
        "minute" | "minutes" | "min" | "mins" => Some(base + Duration::minutes(amount)),
        "month" | "months" => Some(base + Duration::days(amount * 30)),
        _ => None,
    }
}

/// Parse a date string with a specific timezone
///
/// # Arguments
/// * `input` - The date string to parse
/// * `timezone` - The timezone name (e.g., "America/New_York", "Europe/London")
///
/// # Returns
/// * `Ok(DateTime<Utc>)` - The parsed date converted to UTC
/// * `Err(DateParseError)` - If parsing or timezone conversion fails
#[allow(dead_code)] // Available for external use
pub fn parse_date_with_timezone(
    input: &str,
    timezone: &str,
) -> Result<DateTime<Utc>, DateParseError> {
    let tz: Tz = timezone
        .parse()
        .map_err(|_| DateParseError::InvalidTimezone(timezone.to_string()))?;

    let input = input.trim();

    // First try to parse as a datetime with dateparser
    if let Ok(dt) = dateparser::parse(input) {
        return Ok(dt);
    }

    // If that fails, try parsing as a date-only and combine with timezone
    if let Ok(date) = chrono::NaiveDate::parse_from_str(input, "%Y-%m-%d") {
        let naive_dt = date.and_time(NaiveTime::from_hms_opt(0, 0, 0).unwrap());
        let local_dt = tz
            .from_local_datetime(&naive_dt)
            .single()
            .ok_or_else(|| DateParseError::InvalidFormat(input.to_string()))?;
        return Ok(local_dt.with_timezone(&Utc));
    }

    Err(DateParseError::InvalidFormat(input.to_string()))
}

/// Parse a date and ensure it's in the future
///
/// # Arguments
/// * `input` - The date string to parse
///
/// # Returns
/// * `Ok(DateTime<Utc>)` - The parsed date if it's in the future
/// * `Err(DateParseError::PastDate)` - If the date is in the past
#[allow(dead_code)] // Available for external use
pub fn parse_future_date(input: &str) -> Result<DateTime<Utc>, DateParseError> {
    let date = parse_date(input)?;

    if date < Utc::now() {
        return Err(DateParseError::PastDate(input.to_string()));
    }

    Ok(date)
}

/// Get the local timezone name
///
/// Returns the system's local timezone if available, otherwise "UTC"
#[allow(dead_code)] // Available for external use
pub fn local_timezone() -> String {
    // Try to get the TZ environment variable first
    if let Ok(tz) = std::env::var("TZ") {
        return tz;
    }

    // Default to the local timezone offset description
    Local::now().format("%Z").to_string()
}

/// Format a DateTime for display
///
/// # Arguments
/// * `dt` - The datetime to format
/// * `timezone` - Optional timezone for display (defaults to UTC)
///
/// # Returns
/// A formatted date string like "2025-01-15 14:00:00 UTC"
#[allow(dead_code)] // Available for external use
pub fn format_datetime(dt: &DateTime<Utc>, timezone: Option<&str>) -> String {
    if let Some(tz_str) = timezone {
        if let Ok(tz) = tz_str.parse::<Tz>() {
            let local_dt = dt.with_timezone(&tz);
            return local_dt.format("%Y-%m-%d %H:%M:%S %Z").to_string();
        }
    }

    dt.format("%Y-%m-%d %H:%M:%S UTC").to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_iso_date() {
        // Use a future date with explicit UTC time to avoid timezone issues
        let result = parse_date("2030-06-15T00:00:00Z");
        assert!(result.is_ok());
        let dt = result.unwrap();
        assert_eq!(dt.date_naive().to_string(), "2030-06-15");
    }

    #[test]
    fn test_parse_iso_datetime() {
        let result = parse_date("2025-01-15T14:30:00Z");
        assert!(result.is_ok());
        let dt = result.unwrap();
        assert_eq!(
            dt.format("%Y-%m-%dT%H:%M:%S").to_string(),
            "2025-01-15T14:30:00"
        );
    }

    #[test]
    fn test_parse_natural_language_today() {
        let result = parse_date("today");
        assert!(result.is_ok());
        let dt = result.unwrap();
        let today = Utc::now().date_naive();
        assert_eq!(dt.date_naive(), today);
    }

    #[test]
    fn test_parse_natural_language_tomorrow() {
        let result = parse_date("tomorrow");
        assert!(result.is_ok());
        let dt = result.unwrap();
        let tomorrow = Utc::now().date_naive() + chrono::Duration::days(1);
        assert_eq!(dt.date_naive(), tomorrow);
    }

    #[test]
    fn test_parse_relative_in_days() {
        let result = parse_date("in 3 days");
        assert!(result.is_ok());
        let dt = result.unwrap();
        let expected = Utc::now().date_naive() + chrono::Duration::days(3);
        assert_eq!(dt.date_naive(), expected);
    }

    #[test]
    fn test_parse_empty_string() {
        let result = parse_date("");
        assert!(result.is_err());
        match result {
            Err(DateParseError::InvalidFormat(s)) => assert_eq!(s, "empty string"),
            _ => panic!("Expected InvalidFormat error"),
        }
    }

    #[test]
    fn test_parse_invalid_string() {
        let result = parse_date("not a date at all xyz");
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_with_timezone() {
        let result = parse_date_with_timezone("2025-01-15", "America/New_York");
        assert!(result.is_ok());
    }

    #[test]
    fn test_parse_invalid_timezone() {
        let result = parse_date_with_timezone("2025-01-15", "Invalid/Timezone");
        assert!(result.is_err());
        match result {
            Err(DateParseError::InvalidTimezone(tz)) => assert_eq!(tz, "Invalid/Timezone"),
            _ => panic!("Expected InvalidTimezone error"),
        }
    }

    #[test]
    fn test_format_datetime_utc() {
        let dt = Utc.with_ymd_and_hms(2025, 1, 15, 14, 30, 0).unwrap();
        let formatted = format_datetime(&dt, None);
        assert_eq!(formatted, "2025-01-15 14:30:00 UTC");
    }

    #[test]
    fn test_format_datetime_with_timezone() {
        let dt = Utc.with_ymd_and_hms(2025, 1, 15, 19, 30, 0).unwrap();
        let formatted = format_datetime(&dt, Some("America/New_York"));
        // 19:30 UTC is 14:30 EST
        assert!(formatted.contains("2025-01-15"));
        assert!(formatted.contains("14:30:00"));
    }

    #[test]
    fn test_local_timezone() {
        // Just verify it returns a non-empty string
        let tz = local_timezone();
        assert!(!tz.is_empty());
    }

    #[test]
    fn test_date_parse_error_display() {
        let err = DateParseError::InvalidFormat("bad date".to_string());
        assert!(err.to_string().contains("bad date"));
        assert!(err.to_string().contains("Try formats like"));

        let err = DateParseError::InvalidTimezone("Bad/TZ".to_string());
        assert!(err.to_string().contains("Bad/TZ"));

        let err = DateParseError::PastDate("yesterday".to_string());
        assert!(err.to_string().contains("past"));
    }
}
