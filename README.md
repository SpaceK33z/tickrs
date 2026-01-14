# ticktickrs

[![CI](https://github.com/SpaceK33z/tickrs/actions/workflows/ci.yml/badge.svg)](https://github.com/SpaceK33z/tickrs/actions/workflows/ci.yml)
[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](LICENSE)

A command-line interface for [TickTick](https://ticktick.com) app.
`tickrs` provides two modes, normal CLI output and structured JSON output for automation and scripting.

Inspired by [tickli](https://github.com/sho0pi/tickli), but minus the interactive shell and some extra features.

## Features

- Full CRUD operations for projects, tasks, and subtasks
- JSON output mode (`--json`) for AI agents and automation
- Natural language date parsing ("tomorrow", "in 3 days", "next week")
- Quiet mode (`--quiet`) for scripts that only need exit codes
- OAuth 2.0 authentication with secure token storage
- Default project context to reduce repetitive flags

## Installation

### From Releases

Download the latest release for your platform from [GitHub Releases](https://github.com/SpaceK33z/tickrs/releases).

```bash
# macOS (Apple Silicon)
curl -LO https://github.com/SpaceK33z/tickrs/releases/latest/download/tickrs-aarch64-apple-darwin.tar.gz
tar -xzf tickrs-aarch64-apple-darwin.tar.gz
sudo mv tickrs /usr/local/bin/

# macOS (Intel)
curl -LO https://github.com/SpaceK33z/tickrs/releases/latest/download/tickrs-x86_64-apple-darwin.tar.gz
tar -xzf tickrs-x86_64-apple-darwin.tar.gz
sudo mv tickrs /usr/local/bin/

# Linux
curl -LO https://github.com/SpaceK33z/tickrs/releases/latest/download/tickrs-x86_64-unknown-linux-gnu.tar.gz
tar -xzf tickrs-x86_64-unknown-linux-gnu.tar.gz
sudo mv tickrs /usr/local/bin/
```

### With Cargo

```bash
cargo install tickrs
```

### From Source

```bash
git clone https://github.com/SpaceK33z/tickrs.git
cd tickrs
cargo build --release
# Binary will be at ./target/release/tickrs
```

## Quick Start

### 1. Set Up TickTick OAuth App

1. Go to [TickTick Developer Portal](https://developer.ticktick.com/manage)
2. Create a new app
3. Set the redirect URI to `http://localhost:8080`
4. Note your Client ID and Client Secret

### 2. Configure Environment Variables

```bash
export TICKTICK_CLIENT_ID="your_client_id"
export TICKTICK_CLIENT_SECRET="your_client_secret"
```

### 3. Initialize Authentication

```bash
tickrs init
```

This opens your browser for TickTick authorization. After authorizing, the token is stored securely at `~/.local/share/tickrs/token`.

### 4. Start Using tickrs

```bash
# List all projects
tickrs project list

# Set a default project
tickrs project use "Work"

# Create a task
tickrs task create --title "Review PR" --priority high --date "tomorrow"

# List tasks
tickrs task list

# Complete a task
tickrs task complete <task-id>
```

## Command Reference

### Global Options

| Option | Description |
|--------|-------------|
| `--json` | Output in JSON format for machine consumption |
| `-q, --quiet` | Suppress all output (useful for scripts that only need exit codes) |

### Root Commands

#### `tickrs init`
Initialize OAuth authentication with TickTick. Opens browser for authorization.

#### `tickrs reset [--force]`
Clear configuration and stored token. Use `--force` to skip confirmation.

#### `tickrs version`
Display version information.

### Project Commands

#### `tickrs project list` (alias: `ls`)
List all projects.

```bash
tickrs project list
tickrs project list --json
```

#### `tickrs project show <id>`
Show details of a specific project.

```bash
tickrs project show abc123
tickrs project show inbox
```

#### `tickrs project use <name-or-id>`
Set the default project for subsequent commands.

```bash
tickrs project use "Work"
tickrs project use abc123
```

#### `tickrs project create`
Create a new project.

| Option | Description |
|--------|-------------|
| `-n, --name <NAME>` | Project name (required) |
| `-c, --color <COLOR>` | Hex color code (e.g., `#FF5733`) |
| `--view-mode <MODE>` | View mode: `list`, `kanban`, `timeline` |
| `--kind <KIND>` | Project kind: `task`, `note` |

```bash
tickrs project create --name "Side Project" --color "#00AAFF"
```

#### `tickrs project update <id>`
Update an existing project.

| Option | Description |
|--------|-------------|
| `-n, --name <NAME>` | New project name |
| `-c, --color <COLOR>` | New hex color code |
| `--closed` | Archive the project |

```bash
tickrs project update abc123 --name "Archived Project" --closed
```

#### `tickrs project delete <id> [--force]`
Delete a project. Use `--force` to skip confirmation.

```bash
tickrs project delete abc123 --force
```

### Task Commands

#### `tickrs task list` (alias: `ls`)
List tasks in a project.

| Option | Description |
|--------|-------------|
| `-p, --project-id <ID>` | Project ID (uses default if not specified) |
| `--priority <PRIORITY>` | Filter by priority: `none`, `low`, `medium`, `high` |
| `--tag <TAG>` | Filter by tag |
| `--status <STATUS>` | Filter by status: `complete`, `incomplete` |

```bash
tickrs task list
tickrs task list --priority high --status incomplete
tickrs task list --project-id inbox --json
```

#### `tickrs task show <id>`
Show details of a specific task.

```bash
tickrs task show task123
tickrs task show task123 --project-id abc123
```

#### `tickrs task create` (alias: `add`)
Create a new task.

| Option | Description |
|--------|-------------|
| `-t, --title <TITLE>` | Task title (required) |
| `-p, --project-id <ID>` | Project ID (uses default if not specified) |
| `-c, --content <CONTENT>` | Task description |
| `--priority <PRIORITY>` | Priority: `none`, `low`, `medium`, `high` |
| `--tags <TAGS>` | Comma-separated tags |
| `--items <ITEMS>` | Comma-separated subtasks/checklist items |
| `--date <DATE>` | Natural language date (sets start and due) |
| `--start <DATE>` | Start date (ISO 8601) |
| `--due <DATE>` | Due date (ISO 8601) |
| `--all-day` | Mark as all-day task |
| `--timezone <TZ>` | Timezone |

```bash
# Basic task
tickrs task create --title "Buy groceries"

# Task with priority and due date
tickrs task create --title "Submit report" --priority high --date "tomorrow at 5pm"

# Task with tags
tickrs task create --title "Code review" --tags "work,urgent" --date "in 2 days"

# Task in specific project
tickrs task create --title "Research" --project-id abc123 --content "Look into new frameworks"

# Task with subtasks
tickrs task create --title "Pack for trip" --items "Passport,Clothes,Toiletries,Chargers"
```

#### `tickrs task update <id>`
Update an existing task.

```bash
tickrs task update task123 --title "Updated title" --priority medium
tickrs task update task123 --due "2026-01-20T14:00:00Z"
tickrs task update task123 --items "Step 1,Step 2,Step 3"
```

#### `tickrs task delete <id> [--force]`
Delete a task.

```bash
tickrs task delete task123 --force
```

#### `tickrs task complete <id>`
Mark a task as complete.

```bash
tickrs task complete task123
```

#### `tickrs task uncomplete <id>`
Mark a task as incomplete.

```bash
tickrs task uncomplete task123
```

### Subtask Commands

#### `tickrs subtask list <task-id>` (alias: `ls`)
List subtasks (checklist items) for a task.

```bash
tickrs subtask list task123
tickrs subtask list task123 --json
```

> **Note:** To create or modify subtasks, use the `--items` flag on `tickrs task create` or `tickrs task update`. For example: `tickrs task create --title "My task" --items "Step 1,Step 2,Step 3"`

## JSON Output

All commands support `--json` for structured output suitable for AI agents and scripts.

### Success Response

```json
{
  "success": true,
  "data": {
    "tasks": [
      {
        "id": "task123",
        "projectId": "abc123",
        "title": "Complete report",
        "status": 0,
        "priority": 3,
        "dueDate": "2026-01-15T14:00:00Z",
        "tags": ["urgent", "work"]
      }
    ],
    "count": 1
  }
}
```

### Error Response

```json
{
  "success": false,
  "error": {
    "code": "AUTH_REQUIRED",
    "message": "Authentication required. Run 'tickrs init' to authenticate."
  }
}
```

### Error Codes

| Code | Description |
|------|-------------|
| `AUTH_REQUIRED` | Not authenticated, run `tickrs init` |
| `AUTH_EXPIRED` | Token expired, run `tickrs init` again |
| `NOT_FOUND` | Resource not found |
| `INVALID_REQUEST` | Invalid request parameters |
| `RATE_LIMITED` | API rate limit exceeded |
| `SERVER_ERROR` | TickTick server error |
| `NETWORK_ERROR` | Network connection error |
| `NO_PROJECT` | No project specified and no default set |

## Natural Language Dates

The `--date` flag accepts natural language expressions:

| Expression | Result |
|------------|--------|
| `today` | Today at current time |
| `tomorrow` | Tomorrow at current time |
| `yesterday` | Yesterday at current time |
| `next week` | 7 days from now |
| `next month` | 1 month from now |
| `in 3 days` | 3 days from now |
| `in 2 hours` | 2 hours from now |
| `in 30 minutes` | 30 minutes from now |

ISO 8601 dates are also supported: `2026-01-15T14:00:00Z`

## Configuration

### Config File

Location: `~/.config/tickrs/config.toml`

```toml
# Default project for commands without --project-id
default_project_id = "abc123"

# Default color for new projects
default_project_color = "#FF1111"
```

### Token Storage

Location: `~/.local/share/tickrs/token`

The OAuth access token is stored with 0600 permissions (owner read/write only).

### Environment Variables

| Variable | Description |
|----------|-------------|
| `TICKTICK_CLIENT_ID` | OAuth Client ID (required for init) |
| `TICKTICK_CLIENT_SECRET` | OAuth Client Secret (required for init) |
| `RUST_LOG` | Logging level (e.g., `info`, `debug`) |

## Troubleshooting

### "Authentication required" error

Run `tickrs init` to authenticate with TickTick.

### "No project specified" error

Either:
1. Set a default project: `tickrs project use "Project Name"`
2. Specify project explicitly: `tickrs task list --project-id abc123`

### OAuth flow fails to complete

1. Ensure `TICKTICK_CLIENT_ID` and `TICKTICK_CLIENT_SECRET` are set
2. Check that the redirect URI in your TickTick app is `http://localhost:8080`
3. Ensure port 8080 is not in use by another application

### Token expired

Run `tickrs init` again to re-authenticate.

### "Rate limited" error

TickTick API has rate limits. Wait a few minutes before retrying.

## AI Agent Usage

`tickrs` is designed for AI agents and automation. Key features:

1. **JSON output**: Use `--json` for structured, parseable output
2. **Exit codes**: Check `$?` for success (0) or failure (non-zero)
3. **Quiet mode**: Use `--quiet` when you only need exit codes
4. **No interactive prompts**: All commands can run non-interactively with `--force`

Example automation script:

```bash
#!/bin/bash

# Create a task and capture the ID
result=$(tickrs task create --title "Automated task" --json)
task_id=$(echo "$result" | jq -r '.data.task.id')

if [ "$task_id" != "null" ]; then
    echo "Created task: $task_id"
else
    echo "Failed to create task"
    exit 1
fi
```

## Development

```bash
# Build
cargo build

# Run tests
cargo test

# Run with verbose logging
RUST_LOG=debug cargo run -- project list

# Format code
cargo fmt

# Lint
cargo clippy
```

## License

MIT
