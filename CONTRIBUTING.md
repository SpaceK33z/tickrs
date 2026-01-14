# Contributing to tickrs

Thank you for your interest in contributing to tickrs! This document provides guidelines and instructions for contributing.

## Development Setup

### Prerequisites

- Rust 1.70 or later (install via [rustup](https://rustup.rs/))
- Git
- A TickTick account (for integration testing)

### Getting Started

1. **Clone the repository:**
   ```bash
   git clone https://github.com/yourusername/tickrs.git
   cd tickrs
   ```

2. **Build the project:**
   ```bash
   cargo build
   ```

3. **Run tests:**
   ```bash
   cargo test
   ```

4. **Set up environment variables (for integration testing):**
   ```bash
   cp .env.example .env
   # Edit .env with your TickTick OAuth credentials
   ```

### Project Structure

```
tickrs/
├── src/
│   ├── main.rs          # Entry point, CLI command dispatch
│   ├── lib.rs           # Library exports for integration tests
│   ├── constants.rs     # Centralized constants
│   ├── api/             # TickTick API client
│   │   ├── mod.rs
│   │   ├── client.rs    # HTTP client wrapper
│   │   ├── auth.rs      # OAuth flow
│   │   ├── project.rs   # Project endpoints
│   │   └── task.rs      # Task endpoints
│   ├── cli/             # Command definitions (clap)
│   │   ├── mod.rs
│   │   ├── project.rs   # Project subcommands
│   │   ├── task.rs      # Task subcommands
│   │   └── subtask.rs   # Subtask subcommands
│   ├── config/          # Configuration management
│   │   └── mod.rs
│   ├── models/          # Domain models
│   │   ├── mod.rs
│   │   ├── task.rs
│   │   ├── project.rs
│   │   ├── subtask.rs
│   │   ├── priority.rs
│   │   └── status.rs
│   ├── output/          # Output formatting
│   │   ├── mod.rs
│   │   ├── json.rs
│   │   └── text.rs
│   └── utils/           # Utilities
│       ├── mod.rs
│       ├── date_parser.rs
│       └── error.rs
├── tests/               # Integration tests
│   ├── api_integration_tests.rs
│   ├── cli_tests.rs
│   └── oauth_tests.rs
└── plans/               # Project documentation
    └── PRD.md
```

## Code Style Guide

### Formatting

All code must be formatted with `rustfmt`:

```bash
cargo fmt
```

Before committing, verify formatting:

```bash
cargo fmt --check
```

### Linting

All code must pass `clippy` with warnings as errors:

```bash
cargo clippy -- -D warnings
```

### Naming Conventions

- **Modules:** `snake_case` (e.g., `date_parser.rs`)
- **Types/Structs/Enums:** `PascalCase` (e.g., `TaskCommands`)
- **Functions/Methods:** `snake_case` (e.g., `parse_date`)
- **Constants:** `SCREAMING_SNAKE_CASE` (e.g., `API_BASE_URL`)

### Documentation

- All public functions should have doc comments
- Use `///` for item documentation
- Use `//!` for module-level documentation
- Include examples in doc comments where helpful

### Error Handling

- Use `anyhow::Result` for functions that can fail
- Use `thiserror` for custom error types
- Provide user-friendly error messages
- Include error codes for JSON output

### API Design

- All CLI commands must support `--json` output
- JSON responses follow the standard format:
  ```json
  {"success": true, "data": {...}, "message": "..."}
  ```
- Text output should be human-readable and concise

## Testing Guidelines

### Running Tests

```bash
# Run all tests
cargo test

# Run tests with output
cargo test -- --nocapture

# Run specific test
cargo test test_name

# Run tests in specific file
cargo test --test cli_tests
```

### Test Organization

- **Unit tests:** In the same file as the code, in a `#[cfg(test)]` module
- **Integration tests:** In the `tests/` directory

### Writing Tests

1. **Unit tests** should test individual functions in isolation:
   ```rust
   #[cfg(test)]
   mod tests {
       use super::*;

       #[test]
       fn test_function_name() {
           // Arrange
           let input = "test";

           // Act
           let result = function_under_test(input);

           // Assert
           assert_eq!(result, expected);
       }
   }
   ```

2. **Integration tests** should test the CLI or API client end-to-end:
   ```rust
   #[tokio::test]
   async fn test_api_endpoint() {
       // Use wiremock for API tests
       let mock_server = MockServer::start().await;
       // ...
   }
   ```

3. **CLI tests** should use `assert_cmd`:
   ```rust
   #[test]
   fn test_cli_command() {
       let mut cmd = Command::cargo_bin("tickrs").unwrap();
       cmd.arg("version")
          .assert()
          .success()
          .stdout(predicates::str::contains("tickrs"));
   }
   ```

### Test Coverage

- Aim for comprehensive test coverage of public APIs
- Test edge cases (empty lists, special characters, etc.)
- Test error conditions

## Pull Request Process

### Before Submitting

1. **Ensure all tests pass:**
   ```bash
   cargo test
   ```

2. **Check formatting:**
   ```bash
   cargo fmt --check
   ```

3. **Run clippy:**
   ```bash
   cargo clippy -- -D warnings
   ```

4. **Update documentation** if you've changed public APIs

5. **Update `progress.txt`** with a summary of your changes

### Submitting a PR

1. **Fork the repository** and create your branch from `main`

2. **Use a descriptive branch name:**
   - `feature/add-batch-operations`
   - `fix/date-parsing-edge-case`
   - `docs/update-readme`

3. **Write a clear PR description:**
   - What does this PR do?
   - Why is this change needed?
   - How was it tested?

4. **Keep PRs focused:** One feature or fix per PR

### PR Review

- PRs require at least one approval before merging
- Address all review comments
- Keep the PR up to date with the base branch

## Commit Messages

Follow conventional commit format:

```
type(scope): short description

Longer description if needed.

Co-Authored-By: Your Name <email@example.com>
```

Types:
- `feat`: New feature
- `fix`: Bug fix
- `docs`: Documentation only
- `style`: Formatting, no code change
- `refactor`: Code change that neither fixes a bug nor adds a feature
- `test`: Adding tests
- `chore`: Maintenance tasks

Examples:
```
feat(cli): add batch task creation support
fix(api): handle rate limit errors gracefully
docs: update installation instructions
test: add edge case tests for date parsing
```

## Reporting Issues

When reporting issues, please include:

1. **Description:** Clear description of the problem
2. **Steps to reproduce:** Minimal steps to reproduce the issue
3. **Expected behavior:** What you expected to happen
4. **Actual behavior:** What actually happened
5. **Environment:** OS, Rust version, tickrs version
6. **Logs:** Relevant error messages or output (use `--verbose`)

## Feature Requests

For feature requests:

1. **Check existing issues** to avoid duplicates
2. **Describe the use case:** Why is this feature needed?
3. **Propose a solution:** How should it work?
4. **Consider alternatives:** Are there other ways to achieve this?

## Questions?

If you have questions about contributing, feel free to open an issue with the "question" label.

Thank you for contributing to tickrs!
