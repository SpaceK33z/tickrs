//! CLI integration tests for tickrs
//!
//! These tests verify the CLI commands work correctly by executing the actual binary.

use assert_cmd::Command;
use predicates::prelude::*;

const VERSION: &str = env!("CARGO_PKG_VERSION");

// =============================================================================
// Version Command Tests
// =============================================================================

#[test]
fn test_version_command_text_output() {
    let mut cmd = Command::cargo_bin("tickrs").unwrap();
    cmd.arg("version")
        .assert()
        .success()
        .stdout(predicate::str::contains("tickrs"))
        .stdout(predicate::str::contains(VERSION));
}

#[test]
fn test_version_command_json_output() {
    let mut cmd = Command::cargo_bin("tickrs").unwrap();
    let expected_version = format!(r#""version": "{}""#, VERSION);
    cmd.args(["--json", "version"])
        .assert()
        .success()
        .stdout(predicate::str::contains(r#""success": true"#))
        .stdout(predicate::str::contains(r#""name": "ticktickrs""#))
        .stdout(predicate::str::contains(expected_version));
}

#[test]
fn test_version_command_quiet() {
    let mut cmd = Command::cargo_bin("tickrs").unwrap();
    cmd.args(["--quiet", "version"])
        .assert()
        .success()
        .stdout(predicate::str::is_empty());
}

// =============================================================================
// Help Output Tests
// =============================================================================

#[test]
fn test_help_output() {
    let mut cmd = Command::cargo_bin("tickrs").unwrap();
    cmd.arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains("tickrs"))
        .stdout(predicate::str::contains("init"))
        .stdout(predicate::str::contains("reset"))
        .stdout(predicate::str::contains("version"))
        .stdout(predicate::str::contains("project"))
        .stdout(predicate::str::contains("task"))
        .stdout(predicate::str::contains("subtask"));
}

#[test]
fn test_project_help_output() {
    let mut cmd = Command::cargo_bin("tickrs").unwrap();
    cmd.args(["project", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("list"))
        .stdout(predicate::str::contains("show"))
        .stdout(predicate::str::contains("create"))
        .stdout(predicate::str::contains("update"))
        .stdout(predicate::str::contains("delete"))
        .stdout(predicate::str::contains("use"));
}

#[test]
fn test_task_help_output() {
    let mut cmd = Command::cargo_bin("tickrs").unwrap();
    cmd.args(["task", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("list"))
        .stdout(predicate::str::contains("show"))
        .stdout(predicate::str::contains("create"))
        .stdout(predicate::str::contains("update"))
        .stdout(predicate::str::contains("delete"))
        .stdout(predicate::str::contains("complete"))
        .stdout(predicate::str::contains("uncomplete"));
}

#[test]
fn test_subtask_help_output() {
    let mut cmd = Command::cargo_bin("tickrs").unwrap();
    cmd.args(["subtask", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("list"));
}

// =============================================================================
// Reset Command Tests (uses temp directories for isolation)
// =============================================================================

#[test]
fn test_reset_nothing_to_reset_text() {
    // Use a temp directory to ensure clean state
    let temp_dir = tempfile::tempdir().unwrap();

    let mut cmd = Command::cargo_bin("tickrs").unwrap();
    cmd.env("HOME", temp_dir.path())
        .env("XDG_CONFIG_HOME", temp_dir.path().join("config"))
        .env("XDG_DATA_HOME", temp_dir.path().join("data"))
        .args(["reset", "--force"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Nothing to reset"));
}

#[test]
fn test_reset_nothing_to_reset_json() {
    let temp_dir = tempfile::tempdir().unwrap();

    let mut cmd = Command::cargo_bin("tickrs").unwrap();
    cmd.env("HOME", temp_dir.path())
        .env("XDG_CONFIG_HOME", temp_dir.path().join("config"))
        .env("XDG_DATA_HOME", temp_dir.path().join("data"))
        .args(["--json", "reset", "--force"])
        .assert()
        .success()
        .stdout(predicate::str::contains(r#""success": true"#))
        .stdout(predicate::str::contains("Nothing to reset"));
}

// =============================================================================
// Init Command Tests (without actual OAuth)
// =============================================================================

#[test]
fn test_init_missing_client_id() {
    let temp_dir = tempfile::tempdir().unwrap();

    let mut cmd = Command::cargo_bin("tickrs").unwrap();
    cmd.env("HOME", temp_dir.path())
        .env("XDG_CONFIG_HOME", temp_dir.path().join("config"))
        .env("XDG_DATA_HOME", temp_dir.path().join("data"))
        .env_remove("TICKTICK_CLIENT_ID")
        .env_remove("TICKTICK_CLIENT_SECRET")
        .arg("init")
        .assert()
        .failure()
        .stderr(predicate::str::contains("TICKTICK_CLIENT_ID"));
}

#[test]
fn test_init_missing_client_secret() {
    let temp_dir = tempfile::tempdir().unwrap();

    let mut cmd = Command::cargo_bin("tickrs").unwrap();
    cmd.env("HOME", temp_dir.path())
        .env("XDG_CONFIG_HOME", temp_dir.path().join("config"))
        .env("XDG_DATA_HOME", temp_dir.path().join("data"))
        .env("TICKTICK_CLIENT_ID", "test_id")
        .env_remove("TICKTICK_CLIENT_SECRET")
        .arg("init")
        .assert()
        .failure()
        .stderr(predicate::str::contains("TICKTICK_CLIENT_SECRET"));
}

// =============================================================================
// Project Command Tests (without token - should fail gracefully)
// =============================================================================

#[test]
fn test_project_list_no_token_text() {
    let temp_dir = tempfile::tempdir().unwrap();

    let mut cmd = Command::cargo_bin("tickrs").unwrap();
    cmd.env("HOME", temp_dir.path())
        .env("XDG_CONFIG_HOME", temp_dir.path().join("config"))
        .env("XDG_DATA_HOME", temp_dir.path().join("data"))
        .args(["project", "list"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("token").or(predicate::str::contains("authenticate")));
}

#[test]
fn test_project_list_no_token_json() {
    let temp_dir = tempfile::tempdir().unwrap();

    let mut cmd = Command::cargo_bin("tickrs").unwrap();
    cmd.env("HOME", temp_dir.path())
        .env("XDG_CONFIG_HOME", temp_dir.path().join("config"))
        .env("XDG_DATA_HOME", temp_dir.path().join("data"))
        .args(["--json", "project", "list"])
        .assert()
        .failure();
}

#[test]
fn test_project_show_no_token() {
    let temp_dir = tempfile::tempdir().unwrap();

    let mut cmd = Command::cargo_bin("tickrs").unwrap();
    cmd.env("HOME", temp_dir.path())
        .env("XDG_CONFIG_HOME", temp_dir.path().join("config"))
        .env("XDG_DATA_HOME", temp_dir.path().join("data"))
        .args(["project", "show", "test-id"])
        .assert()
        .failure();
}

#[test]
fn test_project_create_requires_name() {
    let mut cmd = Command::cargo_bin("tickrs").unwrap();
    cmd.args(["project", "create"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("--name"));
}

#[test]
fn test_project_use_requires_argument() {
    let mut cmd = Command::cargo_bin("tickrs").unwrap();
    cmd.args(["project", "use"]).assert().failure();
}

#[test]
fn test_project_delete_requires_id() {
    let mut cmd = Command::cargo_bin("tickrs").unwrap();
    cmd.args(["project", "delete"]).assert().failure();
}

// =============================================================================
// Task Command Tests (without token)
// =============================================================================

#[test]
fn test_task_list_no_project() {
    let temp_dir = tempfile::tempdir().unwrap();

    let mut cmd = Command::cargo_bin("tickrs").unwrap();
    cmd.env("HOME", temp_dir.path())
        .env("XDG_CONFIG_HOME", temp_dir.path().join("config"))
        .env("XDG_DATA_HOME", temp_dir.path().join("data"))
        .args(["task", "list"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("project"));
}

#[test]
fn test_task_create_requires_title() {
    let mut cmd = Command::cargo_bin("tickrs").unwrap();
    cmd.args(["task", "create"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("--title"));
}

#[test]
fn test_task_show_requires_id() {
    let mut cmd = Command::cargo_bin("tickrs").unwrap();
    cmd.args(["task", "show"]).assert().failure();
}

#[test]
fn test_task_delete_requires_id() {
    let mut cmd = Command::cargo_bin("tickrs").unwrap();
    cmd.args(["task", "delete"]).assert().failure();
}

#[test]
fn test_task_complete_requires_id() {
    let mut cmd = Command::cargo_bin("tickrs").unwrap();
    cmd.args(["task", "complete"]).assert().failure();
}

#[test]
fn test_task_uncomplete_requires_id() {
    let mut cmd = Command::cargo_bin("tickrs").unwrap();
    cmd.args(["task", "uncomplete"]).assert().failure();
}

#[test]
fn test_task_list_project_name_flag_in_help() {
    let mut cmd = Command::cargo_bin("tickrs").unwrap();
    cmd.args(["task", "list", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("--project-name"))
        .stdout(predicate::str::contains("-n"));
}

#[test]
fn test_task_create_project_name_flag_in_help() {
    let mut cmd = Command::cargo_bin("tickrs").unwrap();
    cmd.args(["task", "create", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("--project-name"))
        .stdout(predicate::str::contains("-n"));
}

#[test]
fn test_task_list_project_id_and_name_conflict() {
    let temp_dir = tempfile::tempdir().unwrap();

    let mut cmd = Command::cargo_bin("tickrs").unwrap();
    cmd.env("HOME", temp_dir.path())
        .env("XDG_CONFIG_HOME", temp_dir.path().join("config"))
        .env("XDG_DATA_HOME", temp_dir.path().join("data"))
        .args([
            "task",
            "list",
            "--project-id",
            "123",
            "--project-name",
            "Test",
        ])
        .assert()
        .failure()
        .stderr(predicate::str::contains("Cannot specify both"));
}

#[test]
fn test_subtask_list_project_name_flag_in_help() {
    let mut cmd = Command::cargo_bin("tickrs").unwrap();
    cmd.args(["subtask", "list", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("--project-name"))
        .stdout(predicate::str::contains("-n"));
}

// =============================================================================
// Subtask Command Tests
// =============================================================================

#[test]
fn test_subtask_list_requires_task_id() {
    let mut cmd = Command::cargo_bin("tickrs").unwrap();
    cmd.args(["subtask", "list"]).assert().failure();
}

// =============================================================================
// Global Flags Tests
// =============================================================================

#[test]
fn test_json_flag_position_before_command() {
    let mut cmd = Command::cargo_bin("tickrs").unwrap();
    cmd.args(["--json", "version"])
        .assert()
        .success()
        .stdout(predicate::str::contains(r#""success": true"#));
}

#[test]
fn test_verbose_flag() {
    let mut cmd = Command::cargo_bin("tickrs").unwrap();
    cmd.args(["--verbose", "version"]).assert().success();
}

#[test]
fn test_quiet_flag() {
    let mut cmd = Command::cargo_bin("tickrs").unwrap();
    cmd.args(["--quiet", "version"])
        .assert()
        .success()
        .stdout(predicate::str::is_empty());
}

#[test]
fn test_short_verbose_flag() {
    let mut cmd = Command::cargo_bin("tickrs").unwrap();
    cmd.args(["-v", "version"]).assert().success();
}

#[test]
fn test_short_quiet_flag() {
    let mut cmd = Command::cargo_bin("tickrs").unwrap();
    cmd.args(["-q", "version"])
        .assert()
        .success()
        .stdout(predicate::str::is_empty());
}

// =============================================================================
// Command Alias Tests
// =============================================================================

#[test]
fn test_project_list_alias_ls() {
    let temp_dir = tempfile::tempdir().unwrap();

    let mut cmd = Command::cargo_bin("tickrs").unwrap();
    cmd.env("HOME", temp_dir.path())
        .env("XDG_CONFIG_HOME", temp_dir.path().join("config"))
        .env("XDG_DATA_HOME", temp_dir.path().join("data"))
        .args(["project", "ls"])
        .assert()
        .failure(); // Fails due to no token, but alias should be recognized
}

#[test]
fn test_task_create_alias_add() {
    let mut cmd = Command::cargo_bin("tickrs").unwrap();
    cmd.args(["task", "add"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("--title")); // Alias recognized, fails due to missing title
}

// =============================================================================
// Invalid Input Tests
// =============================================================================

#[test]
fn test_invalid_command() {
    let mut cmd = Command::cargo_bin("tickrs").unwrap();
    cmd.arg("nonexistent").assert().failure();
}

#[test]
fn test_invalid_project_subcommand() {
    let mut cmd = Command::cargo_bin("tickrs").unwrap();
    cmd.args(["project", "nonexistent"]).assert().failure();
}

#[test]
fn test_invalid_task_subcommand() {
    let mut cmd = Command::cargo_bin("tickrs").unwrap();
    cmd.args(["task", "nonexistent"]).assert().failure();
}

// =============================================================================
// Exit Code Tests
// =============================================================================

#[test]
fn test_success_exit_code() {
    let mut cmd = Command::cargo_bin("tickrs").unwrap();
    cmd.arg("version").assert().code(0);
}

#[test]
fn test_failure_exit_code_invalid_command() {
    let mut cmd = Command::cargo_bin("tickrs").unwrap();
    cmd.arg("nonexistent").assert().code(predicate::ne(0));
}

#[test]
fn test_failure_exit_code_missing_required_arg() {
    let mut cmd = Command::cargo_bin("tickrs").unwrap();
    cmd.args(["task", "create"]).assert().code(predicate::ne(0));
}
