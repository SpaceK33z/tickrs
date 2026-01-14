# Product Requirements Document: Tickrs
## TickTick CLI Refactor - Go to Rust

**Version:** 1.0
**Date:** 2026-01-13
**Status:** Draft

---

## Executive Summary

Refactor the existing `tickli` CLI tool from Go (~2,700 LOC) to Rust, creating `tickrs` - an AI agent-optimized command-line interface for TickTick task management. The refactor will eliminate TUI/interactive components and add comprehensive JSON output modes to enable programmatic consumption by AI agents and automation tools.

### Key Objectives

- **Remove TUI Components**: Strip all interactive user interfaces and fuzzy selectors
- **Add JSON Output**: Implement `--json` flag for all commands with structured output
- **Add Quiet Mode**: Implement `--quiet` flag to suppress output (useful for scripts that only need exit codes)
- **Support Stdin Input**: Allow reading task data from stdin for batch/piped operations
- **Maintain Core Functionality**: Preserve all task and project management capabilities
- **Optimize for Automation**: Design for AI agents and scripting, not human interaction

---

## Source Codebase Analysis

### Architecture Overview

**Original Repository:** `../tickli` (Go implementation)

#### Directory Structure
```
tickli/
├── cmd/                      # Command definitions
│   ├── cmd.go               # Root command with cobra
│   ├── init.go              # OAuth authentication
│   ├── reset.go             # Reset configuration
│   ├── version.go           # Version command
│   ├── project/             # Project commands
│   │   ├── cmd.go
│   │   ├── create.go
│   │   ├── delete.go
│   │   ├── list.go
│   │   ├── show.go
│   │   ├── update.go
│   │   └── use.go
│   ├── task/                # Task commands
│   │   ├── cmd.go
│   │   ├── create.go
│   │   ├── complete.go
│   │   ├── uncomplete.go
│   │   ├── delete.go
│   │   ├── list.go
│   │   ├── show.go
│   │   └── update.go
│   └── subtask/             # Subtask commands
│       ├── cmd.go
│       ├── list.go
│       └── subtask.go
├── internal/
│   ├── api/                 # TickTick API client
│   │   ├── client.go        # HTTP client wrapper
│   │   ├── models.go
│   │   ├── types.go
│   │   └── projects.go
│   ├── config/              # Configuration management
│   │   └── config.go        # Config & token storage
│   ├── types/               # Data structures
│   │   ├── task.go
│   │   ├── project.go
│   │   ├── ticktick_time.go
│   │   ├── output_format.go
│   │   ├── task/
│   │   │   ├── priority.go  # Priority enum (0,1,3,5)
│   │   │   └── status.go    # Status enum (0=normal, 2=complete)
│   │   └── project/
│   │       ├── color.go
│   │       ├── kind.go
│   │       └── view_mode.go
│   ├── utils/               # Utility functions
│   │   ├── client.go
│   │   ├── time.go          # Natural language date parsing
│   │   └── utils.go
│   └── completion/          # Shell completion
│       └── completion.go
└── main.go                  # Entry point
```

#### Key Dependencies (Go)
- `cobra` - CLI framework
- `resty` - HTTP client
- `viper` - Configuration management
- `zerolog` - Logging
- `gookit/color` - Terminal colors (TUI - to be removed)
- `godotenv` - Environment variables

### Core Data Models

#### Task
```go
type Task struct {
    ID            string          // Unique identifier
    ProjectID     string          // Parent project
    Title         string          // Task name
    IsAllDay      bool            // All-day task flag
    CompletedTime TickTickTime    // Completion timestamp
    Content       string          // Description/notes
    Desc          string          // Deprecated - use Content
    DueDate       TickTickTime    // Due date
    Items         []ChecklistItem // Subtasks/checklist
    Priority      Priority        // 0=none, 1=low, 3=medium, 5=high
    Reminders     []string        // Reminder triggers
    RepeatFlag    string          // Recurrence rule
    SortOrder     int64           // Display order
    StartDate     TickTickTime    // Start date
    Status        Status          // 0=normal, 2=complete
    TimeZone      string          // Timezone
    Tags          []string        // Tags for categorization
}
```

#### ChecklistItem (Subtask)
```go
type ChecklistItem struct {
    ID            string
    Title         string
    Status        int             // Completion status
    CompletedTime int64           // Unix timestamp
    IsAllDay      bool
    SortOrder     int64
    StartDate     TickTickTime
    TimeZone      string
}
```

#### Project
```go
type Project struct {
    ID         string           // Unique identifier
    Name       string           // Project name
    Color      Color            // Hex color code
    SortOrder  int64            // Display order
    Closed     bool             // Archive status
    GroupID    string           // Group/folder
    ViewMode   ViewMode         // list/kanban/timeline
    Permission string           // Access permissions
    Kind       Kind             // task/note type
}
```

#### ProjectData
```go
type ProjectData struct {
    Project Project
    Tasks   []Task
    Columns []Column  // Kanban columns
}
```

#### Enums
- **Priority**: `0` (none), `1` (low), `3` (medium), `5` (high)
- **Status**: `0` (normal/incomplete), `2` (complete)
- **ViewMode**: list, kanban, timeline
- **Kind**: task, note

### API Integration

#### TickTick API Endpoints

**Base URL:** `https://api.ticktick.com/open/v1`

**Authentication:**
- OAuth 2.0 flow
- Authorization URL: `https://ticktick.com/oauth/authorize`
- Token URL: `https://ticktick.com/oauth/token`
- Scope: `tasks:write tasks:read`
- Redirect URI: `http://localhost:8080`

**API Methods:**
- `GET /project` - List all projects
- `GET /project/{id}` - Get project details
- `GET /project/{id}/data` - Get project with tasks and columns
- `POST /project` - Create project
- `POST /project/{id}` - Update project
- `DELETE /project/{id}` - Delete project
- `GET /project/{projectId}/task/{taskId}` - Get task
- `POST /task` - Create task
- `POST /task/{id}` - Update task
- `DELETE /project/{projectId}/task/{taskId}` - Delete task
- `POST /project/{projectId}/task/{taskId}/complete` - Mark complete

**Special Handling:**
- Inbox project has ID `"inbox"` and is not returned by the API (must be added manually)
- Bearer token authentication: `Authorization: Bearer {token}`

### Configuration & Storage

**Config File:** `~/.config/tickli/config.yaml`
```yaml
default_project_id: ""        # Currently active project
default_project_color: "#FF1111"  # Default color
```

**Token Storage:** `~/.local/share/tickli/token` (0600 permissions)

### Commands to Port

#### Root Commands
- `init` - OAuth authentication flow
- `reset` - Clear configuration and token
- `version` - Display version information

#### Project Commands (`project` subcommand)
- `project list` - List all projects
- `project show [id]` - Show project details
- `project use [name]` - Set default project (save to config)
- `project create` - Create new project
- `project update [id]` - Update project properties
- `project delete [id]` - Delete project

#### Task Commands (`task` subcommand)
- `task list` - List tasks in project
- `task show [id]` - Show task details
- `task create` - Create new task
- `task update [id]` - Update task
- `task delete [id]` - Delete task
- `task complete [id]` - Mark task complete
- `task uncomplete [id]` - Mark task incomplete

#### Subtask Commands (`subtask` subcommand)
- `subtask list [task-id]` - List subtasks (checklist items)

### Features to Exclude (TUI/Interactive)

**Remove all interactive components:**
- ❌ Fuzzy selectors (`utils.FuzzySelectTask`, `utils.FuzzySelectProject`)
- ❌ Interactive prompts for input
- ❌ Colored terminal output (beyond basic success/error)
- ❌ Terminal UI frameworks (bubbletea references in roadmap)
- ❌ ASCII art logo display
- ❌ Interactive date/time pickers
- ❌ Progress bars or spinners

**Replacement Strategy:**
- Use direct ID/name arguments instead of interactive selection
- Return JSON or plain text output
- Require all parameters via flags/arguments
- Provide clear error messages for missing parameters

### Features to Preserve

**Core Functionality:**
- ✅ OAuth authentication flow (open browser, capture callback)
- ✅ Full CRUD operations for projects and tasks
- ✅ Natural language date parsing (e.g., "tomorrow at 2pm")
- ✅ Timezone support
- ✅ Priority levels (none, low, medium, high)
- ✅ Tags and filtering
- ✅ Subtask/checklist management
- ✅ Configuration persistence
- ✅ Token storage and refresh
- ✅ Default project context

---

## Rust Implementation Plan

### Technology Stack

#### Core Dependencies
- **`clap`** (v4.5.54) - CLI argument parsing with derive macros
- **`tokio`** - Async runtime
- **`reqwest`** - HTTP client with async support
- **`serde`** / **`serde_json`** - Serialization/deserialization
- **`chrono`** - Date/time handling
- **`chrono-tz`** - Timezone support
- **`anyhow`** / **`thiserror`** - Error handling
- **`dirs`** - Standard directory paths (~/.config, ~/.local/share)
- **`toml`** - Config file parsing (using TOML instead of YAML)
- **`dotenv`** - Environment variable loading

#### Additional Dependencies
- **`dateparser`** or **`chrono-english`** - Natural language date parsing
- **`oauth2`** - OAuth 2.0 flow implementation
- **`tokio-stream`** - Stream utilities
- **`tracing`** / **`tracing-subscriber`** - Structured logging

### Project Structure

```
tickrs/
├── Cargo.toml
├── Cargo.lock
├── README.md
├── .env.example              # Example environment variables
├── src/
│   ├── main.rs              # Entry point, CLI setup
│   ├── lib.rs               # Library exports
│   ├── cli/                 # Command definitions
│   │   ├── mod.rs
│   │   ├── root.rs          # Root commands (init, reset, version)
│   │   ├── project.rs       # Project subcommands
│   │   ├── task.rs          # Task subcommands
│   │   └── subtask.rs       # Subtask subcommands
│   ├── api/                 # TickTick API client
│   │   ├── mod.rs
│   │   ├── client.rs        # HTTP client wrapper
│   │   ├── auth.rs          # OAuth flow
│   │   ├── project.rs       # Project endpoints
│   │   ├── task.rs          # Task endpoints
│   │   └── types.rs         # Request/response types
│   ├── models/              # Domain models
│   │   ├── mod.rs
│   │   ├── task.rs
│   │   ├── project.rs
│   │   ├── subtask.rs
│   │   ├── priority.rs
│   │   ├── status.rs
│   │   └── time.rs
│   ├── config/              # Configuration management
│   │   ├── mod.rs
│   │   └── storage.rs
│   ├── output/              # Output formatting
│   │   ├── mod.rs
│   │   ├── json.rs
│   │   └── text.rs
│   ├── utils/               # Utilities
│   │   ├── mod.rs
│   │   ├── date_parser.rs   # Natural language dates
│   │   └── error.rs         # Custom error types
│   └── constants.rs         # Constants (API URLs, defaults)
├── tests/                   # Integration tests
│   ├── api_tests.rs
│   ├── cli_tests.rs
│   └── fixtures/
└── examples/                # Usage examples
    └── automation.rs
```

### Core Types (Rust)

```rust
// models/priority.rs
#[derive(Debug, Clone, Copy, Serialize, Deserialize, ValueEnum)]
#[serde(from = "i32", into = "i32")]
pub enum Priority {
    None = 0,
    Low = 1,
    Medium = 3,
    High = 5,
}

// models/status.rs
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(from = "i32", into = "i32")]
pub enum Status {
    Normal = 0,
    Complete = 2,
}

// models/task.rs
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Task {
    pub id: String,
    pub project_id: String,
    pub title: String,
    pub is_all_day: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub completed_time: Option<DateTime<Utc>>,
    pub content: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub due_date: Option<DateTime<Utc>>,
    pub items: Vec<ChecklistItem>,
    pub priority: Priority,
    pub reminders: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub repeat_flag: Option<String>,
    pub sort_order: i64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub start_date: Option<DateTime<Utc>>,
    pub status: Status,
    pub time_zone: String,
    pub tags: Vec<String>,
}

// models/project.rs
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Project {
    pub id: String,
    pub name: String,
    pub color: String,
    pub sort_order: i64,
    pub closed: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub group_id: Option<String>,
    pub view_mode: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub permission: Option<String>,
    pub kind: String,
}

// models/subtask.rs
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ChecklistItem {
    pub id: String,
    pub title: String,
    pub status: i32,
    pub completed_time: i64,
    pub is_all_day: bool,
    pub sort_order: i64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub start_date: Option<DateTime<Utc>>,
    pub time_zone: String,
}
```

### Output Format Specification

#### JSON Output Mode

All commands must support `--json` flag for machine-readable output.

**Success Response Format:**
```json
{
  "success": true,
  "data": {
    // Command-specific data
  },
  "message": "Operation completed successfully"
}
```

**Error Response Format:**
```json
{
  "success": false,
  "error": {
    "code": "ERROR_CODE",
    "message": "Human-readable error message",
    "details": {} // Optional additional context
  }
}
```

#### Command-Specific JSON Outputs

**`project list --json`**
```json
{
  "success": true,
  "data": {
    "projects": [
      {
        "id": "abc123",
        "name": "Work",
        "color": "#FF5733",
        "closed": false,
        "sortOrder": 0,
        "viewMode": "list",
        "kind": "task"
      }
    ]
  }
}
```

**`task list --json`**
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

**`task create --json`**
```json
{
  "success": true,
  "data": {
    "task": {
      "id": "new_task_id",
      "projectId": "abc123",
      "title": "New task",
      "status": 0
    }
  },
  "message": "Task created successfully"
}
```

#### Text Output Mode (Default)

**List Commands:**
```
Projects:
- [abc123] Work (#FF5733)
- [def456] Personal (#00AAFF)
- [inbox] 📥Inbox

Total: 3 projects
```

**Show Commands:**
```
Task: task123
Title: Complete report
Project: abc123
Status: incomplete
Priority: medium
Due: 2026-01-15 14:00:00 UTC
Tags: urgent, work
```

**Create/Update Commands:**
```
✓ Task created successfully
  ID: task123
```

---

## Implementation Phases

### Phase 1: Project Setup & Foundation
**Goal:** Bootstrap Rust project with core dependencies and architecture

- [ ] Initialize Cargo project with workspace structure
- [ ] Set up dependencies in `Cargo.toml`
  - [ ] Add `clap` with derive features
  - [ ] Add `tokio` with full features
  - [ ] Add `reqwest` with json and rustls-tls
  - [ ] Add `serde`, `serde_json`
  - [ ] Add `chrono`, `chrono-tz`
  - [ ] Add `anyhow`, `thiserror`
  - [ ] Add `dirs` for config paths
  - [ ] Add `toml` for config parsing
  - [ ] Add `tracing`, `tracing-subscriber`
- [ ] Create project directory structure
  - [ ] `src/cli/` module
  - [ ] `src/api/` module
  - [ ] `src/models/` module
  - [ ] `src/config/` module
  - [ ] `src/output/` module
  - [ ] `src/utils/` module
- [ ] Set up `main.rs` with basic CLI skeleton using clap
- [ ] Configure logging with tracing
- [ ] Set up `.env.example` for client ID/secret
- [ ] Create `README.md` with project overview

**Deliverables:**
- Working Rust project that compiles
- Basic CLI help output (`tickrs --help`)
- Module structure in place

---

### Phase 2: Configuration & Storage
**Goal:** Implement configuration management and secure token storage

- [ ] Define configuration structure (`config/mod.rs`)
  - [ ] Config file path: `~/.config/tickrs/config.toml`
  - [ ] Token file path: `~/.local/share/tickrs/token`
  - [ ] Create directories with appropriate permissions
- [ ] Implement `Config` struct
  - [ ] `default_project_id: Option<String>`
  - [ ] `default_project_color: String`
- [ ] Implement config operations
  - [ ] `load()` - Read config from file
  - [ ] `save()` - Write config to file
  - [ ] `init_default()` - Create default config
- [ ] Implement token operations
  - [ ] `load_token()` - Read token from file
  - [ ] `save_token(token: String)` - Write token with 0600 permissions
  - [ ] `delete_token()` - Remove token file
- [ ] Add error handling for file operations
- [ ] Write unit tests for config module

**Deliverables:**
- Config file can be created, read, and written
- Token can be securely stored and retrieved
- Test coverage for config operations

---

### Phase 3: Data Models
**Goal:** Define all core data structures with serde support

- [ ] Implement `models/priority.rs`
  - [ ] Define `Priority` enum (None=0, Low=1, Medium=3, High=5)
  - [ ] Implement `From<i32>` and `Into<i32>`
  - [ ] Implement serde serialization/deserialization
  - [ ] Implement `FromStr` for CLI parsing
  - [ ] Add unit tests
- [ ] Implement `models/status.rs`
  - [ ] Define `Status` enum (Normal=0, Complete=2)
  - [ ] Implement serde support
  - [ ] Add unit tests
- [ ] Implement `models/subtask.rs`
  - [ ] Define `ChecklistItem` struct
  - [ ] Add serde attributes with camelCase
  - [ ] Add unit tests for JSON parsing
- [ ] Implement `models/task.rs`
  - [ ] Define `Task` struct with all fields
  - [ ] Use `Option<T>` for nullable fields
  - [ ] Configure serde with camelCase and skip_serializing_if
  - [ ] Add builder pattern for task creation
  - [ ] Add unit tests with sample JSON
- [ ] Implement `models/project.rs`
  - [ ] Define `Project` struct
  - [ ] Define `ProjectData` struct (project + tasks + columns)
  - [ ] Add INBOX constant
  - [ ] Add unit tests
- [ ] Implement `models/time.rs`
  - [ ] TickTick time format handling
  - [ ] Timezone conversions
  - [ ] Add parsing utilities

**Deliverables:**
- All models can be serialized to/from JSON
- Models match TickTick API format exactly
- Comprehensive unit test coverage

---

### Phase 4: API Client Foundation
**Goal:** Build HTTP client for TickTick API with authentication

- [ ] Implement `api/client.rs`
  - [ ] Define `TickTickClient` struct with reqwest client
  - [ ] Add base URL constant: `https://api.ticktick.com/open/v1`
  - [ ] Implement constructor with bearer token
  - [ ] Add request wrapper with error handling
  - [ ] Add logging for all API calls
- [ ] Implement `api/auth.rs`
  - [ ] OAuth URLs as constants
  - [ ] Implement `get_auth_url(client_id: &str) -> String`
  - [ ] Implement OAuth callback server (localhost:8080)
  - [ ] Implement `exchange_code(client_id, client_secret, code) -> Result<String>`
  - [ ] Handle browser opening with `webbrowser` crate
- [ ] Add custom error types (`utils/error.rs`)
  - [ ] `ApiError` for HTTP errors
  - [ ] `AuthError` for OAuth errors
  - [ ] `ConfigError` for config operations
  - [ ] Implement `Display` and `Error` traits
- [ ] Write integration test skeleton

**Deliverables:**
- HTTP client can make authenticated requests
- OAuth flow can obtain access token
- Error handling covers common scenarios

---

### Phase 5: API Client - Project Endpoints
**Goal:** Implement all project-related API calls

- [ ] Implement `api/project.rs`
  - [ ] `list_projects() -> Result<Vec<Project>>`
    - [ ] GET /project
    - [ ] Append INBOX project to results
  - [ ] `get_project(id: &str) -> Result<Project>`
    - [ ] GET /project/{id}
    - [ ] Handle INBOX special case
  - [ ] `get_project_data(id: &str) -> Result<ProjectData>`
    - [ ] GET /project/{id}/data
  - [ ] `create_project(project: &Project) -> Result<Project>`
    - [ ] POST /project
  - [ ] `update_project(id: &str, project: &Project) -> Result<Project>`
    - [ ] POST /project/{id}
  - [ ] `delete_project(id: &str) -> Result<()>`
    - [ ] DELETE /project/{id}
- [ ] Add logging for each endpoint
- [ ] Write integration tests (requires .env with test credentials)

**Deliverables:**
- All project CRUD operations working
- Integration tests passing
- Error handling for API failures

---

### Phase 6: API Client - Task Endpoints
**Goal:** Implement all task-related API calls

- [ ] Implement `api/task.rs`
  - [ ] `list_tasks(project_id: &str) -> Result<Vec<Task>>`
    - [ ] GET /project/{projectId}/data, extract tasks
  - [ ] `get_task(project_id: &str, task_id: &str) -> Result<Task>`
    - [ ] GET /project/{projectId}/task/{taskId}
  - [ ] `create_task(task: &Task) -> Result<Task>`
    - [ ] POST /task
  - [ ] `update_task(task_id: &str, task: &Task) -> Result<Task>`
    - [ ] POST /task/{id}
  - [ ] `delete_task(project_id: &str, task_id: &str) -> Result<()>`
    - [ ] DELETE /project/{projectId}/task/{taskId}
  - [ ] `complete_task(project_id: &str, task_id: &str) -> Result<()>`
    - [ ] POST /project/{projectId}/task/{taskId}/complete
  - [ ] `uncomplete_task(project_id: &str, task_id: &str) -> Result<()>`
    - [ ] Update task status to 0
- [ ] Add logging and error handling
- [ ] Write integration tests

**Deliverables:**
- All task operations functional
- Task completion/uncompletion working
- Integration tests passing

---

### Phase 7: Output Formatting
**Goal:** Create JSON and text output formatters

- [ ] Implement `output/json.rs`
  - [ ] Define `JsonResponse<T>` wrapper struct
    - [ ] `success: bool`
    - [ ] `data: Option<T>`
    - [ ] `error: Option<ErrorDetail>`
    - [ ] `message: Option<String>`
  - [ ] Implement `to_json_output<T: Serialize>(result: Result<T>) -> String`
  - [ ] Implement success response builder
  - [ ] Implement error response builder
  - [ ] Pretty print with indent=2
- [ ] Implement `output/text.rs`
  - [ ] Format project list for text output
  - [ ] Format task list for text output
  - [ ] Format single project details
  - [ ] Format single task details
  - [ ] Format success/error messages
  - [ ] Add minimal color coding (green=success, red=error)
- [ ] Add `OutputFormat` enum (Json, Text)
- [ ] Implement output dispatcher

**Deliverables:**
- JSON output matches specification
- Text output is human-readable
- Both formats handle all data types

---

### Phase 8: Utilities
**Goal:** Implement date parsing and helper utilities

- [ ] Implement `utils/date_parser.rs`
  - [ ] Integrate natural language date parsing library
  - [ ] Parse expressions like:
    - [ ] "today", "tomorrow", "next week"
    - [ ] "tomorrow at 2pm"
    - [ ] "in 3 days"
    - [ ] ISO 8601 formats
  - [ ] Return `Result<DateTime<Utc>>`
  - [ ] Support timezone specification
  - [ ] Write comprehensive tests for date parsing
- [ ] Implement `utils/error.rs` refinements
  - [ ] User-friendly error messages
  - [ ] Error code enum
  - [ ] Conversion from API errors
- [ ] Add `constants.rs`
  - [ ] API base URL
  - [ ] OAuth URLs
  - [ ] Default config values
  - [ ] INBOX project definition

**Deliverables:**
- Natural language dates parse correctly
- Error messages are clear and actionable
- Constants are centralized

---

### Phase 9: CLI - Root Commands
**Goal:** Implement init, reset, version commands

- [ ] Implement `cli/root.rs`
  - [ ] Define root `Cli` struct with clap
    - [ ] Global `--json` flag
    - [ ] Global `--verbose` flag
  - [ ] **`init` command**
    - [ ] Check if already initialized (token exists)
    - [ ] Load client ID/secret from env
    - [ ] Call OAuth flow (get_auth_url → open browser → capture code)
    - [ ] Exchange code for token
    - [ ] Save token with save_token()
    - [ ] Initialize config file
    - [ ] Output success message (text or JSON)
  - [ ] **`reset` command**
    - [ ] Delete token file
    - [ ] Delete config file
    - [ ] Confirm with user (skip with --force)
    - [ ] Output success message
  - [ ] **`version` command**
    - [ ] Display version from Cargo.toml
    - [ ] Support --json output
- [ ] Wire up commands in `main.rs`
- [ ] Add error handling for each command
- [ ] Write CLI integration tests

**Deliverables:**
- `tickrs init` successfully authenticates
- `tickrs reset` clears configuration
- `tickrs version` displays version
- Both text and JSON output work

---

### Phase 10: CLI - Project Commands
**Goal:** Implement all project subcommands

- [x] Implement `cli/project.rs`
  - [x] Define `ProjectCommands` enum
  - [x] **`project list`**
    - [x] Call `api.list_projects()`
    - [x] Support `--json` output
    - [x] Text: display as table with ID, name, color
  - [x] **`project show <id>`**
    - [x] Positional `<id>` argument
    - [x] Call `api.get_project(id)`
    - [x] Support `--json` output
    - [x] Text: display all project fields
  - [x] **`project use <name-or-id>`**
    - [x] Find project by name or ID
    - [x] Update config.default_project_id
    - [x] Save config
    - [x] Output success message
  - [x] **`project create`**
    - [x] Flags: `--name`, `--color`, `--view-mode`, `--kind`
    - [x] Build Project struct
    - [x] Call `api.create_project()`
    - [x] Output created project
  - [x] **`project update <id>`**
    - [x] Flags: `--name`, `--color`, `--closed`
    - [x] Get existing project, merge changes
    - [x] Call `api.update_project()`
    - [x] Output updated project
  - [x] **`project delete <id>`**
    - [x] Confirm deletion (skip with --force)
    - [x] Call `api.delete_project(id)`
    - [x] Output success message
- [x] Add command aliases (e.g., `ls` for list)
- [ ] Write CLI integration tests for each command

**Deliverables:**
- [x] All project commands functional
- [x] JSON output works for all commands
- [x] Error handling for missing projects

---

### Phase 11: CLI - Task Commands
**Goal:** Implement all task subcommands

- [x] Implement `cli/task.rs`
  - [x] Define `TaskCommands` enum
  - [x] **`task list`**
    - [x] Optional `--project-id` or use default from config
    - [x] Optional filters: `--priority`, `--tag`, `--status`
    - [x] Call `api.list_tasks(project_id)`
    - [x] Apply filters locally
    - [x] Support `--json` output
    - [x] Text: display as table with ID, title, status, priority, due
  - [x] **`task show <id>`**
    - [x] Positional `<id>` argument
    - [x] Optional `--project-id` or use default
    - [x] Call `api.get_task(project_id, id)`
    - [x] Support `--json` output
    - [x] Text: display all task fields including subtasks
  - [x] **`task create`**
    - [x] Required: `--title`
    - [x] Optional flags:
      - [x] `--project-id` (or use default)
      - [x] `--content` (description)
      - [x] `--priority <none|low|medium|high>`
      - [x] `--tags <tag1,tag2>`
      - [x] `--date <natural-language>` (start + due)
      - [x] `--start <ISO-8601>`
      - [x] `--due <ISO-8601>`
      - [x] `--all-day`
      - [x] `--timezone`
    - [x] Parse date with date_parser
    - [x] Build Task struct
    - [x] Call `api.create_task()`
    - [x] Output created task
  - [x] **`task update <id>`**
    - [x] Same flags as create (except --title is optional)
    - [x] Get existing task, merge changes
    - [x] Call `api.update_task()`
    - [x] Output updated task
  - [x] **`task delete <id>`**
    - [x] Optional `--project-id`
    - [x] Confirm deletion (skip with --force)
    - [x] Call `api.delete_task()`
    - [x] Output success message
  - [x] **`task complete <id>`**
    - [x] Optional `--project-id`
    - [x] Call `api.complete_task()`
    - [x] Output success message
  - [x] **`task uncomplete <id>`**
    - [x] Optional `--project-id`
    - [x] Call `api.uncomplete_task()`
    - [x] Output success message
- [x] Add command aliases (e.g., `add` for create)
- [ ] Write CLI integration tests

**Deliverables:**
- [x] All task CRUD operations work
- [x] Date parsing integrates correctly
- [x] Filters work as expected
- [x] JSON output for all commands

---

### Phase 12: CLI - Subtask Commands
**Goal:** Implement subtask listing and management

- [x] Implement `cli/subtask.rs`
  - [x] Define `SubtaskCommands` enum
  - [x] **`subtask list <task-id>`**
    - [x] Optional `--project-id`
    - [x] Call `api.get_task()` to get task with items
    - [x] Extract `items` (checklist)
    - [x] Support `--json` output
    - [x] Text: display as table with ID, title, status
- [x] Note: Subtask create/update/delete would require full task update
  - [ ] Document this limitation in README
  - [ ] Optionally implement as task update with modified items array
- [ ] Write integration tests

**Deliverables:**
- [x] Subtask listing works
- [x] JSON output includes all subtask fields

---

### Phase 13: Testing & Quality Assurance
**Goal:** Comprehensive testing and error handling

- [x] Write unit tests for all modules
  - [x] Models: JSON serialization/deserialization
  - [x] Config: File operations
  - [x] Utils: Date parsing edge cases
  - [x] Output: JSON/text formatting
- [x] Write integration tests
  - [x] API client with mock server
  - [x] CLI commands with test fixtures
  - [ ] OAuth flow simulation
- [x] Add error handling coverage
  - [x] Network errors
  - [x] Invalid JSON responses
  - [x] Missing configuration
  - [x] Invalid token
  - [x] API rate limits
- [x] Test edge cases
  - [x] Empty project/task lists
  - [x] INBOX project handling
  - [x] Timezone conversions
  - [x] Special characters in titles
- [x] Add CI/CD pipeline (GitHub Actions)
  - [x] Cargo fmt check
  - [x] Cargo clippy
  - [x] Cargo test
  - [x] Build for multiple platforms

**Deliverables:**
- >80% test coverage
- All tests passing in CI
- Error messages are helpful

---

### Phase 14: Documentation & Examples
**Goal:** Complete user and developer documentation

- [ ] Write comprehensive README.md
  - [ ] Installation instructions
  - [ ] Quick start guide
  - [ ] Authentication setup
  - [ ] Command reference with examples
  - [ ] JSON output examples
  - [ ] Troubleshooting section
- [ ] Create `.env.example`
  - [ ] Document required environment variables
  - [ ] Add instructions for TickTick OAuth app setup
- [ ] Write CONTRIBUTING.md
  - [ ] Development setup
  - [ ] Code style guide
  - [ ] Testing guidelines
  - [ ] Pull request process
- [ ] Add inline documentation
  - [ ] All public functions have doc comments
  - [ ] Module-level documentation
  - [ ] Example code in doc comments
- [ ] Create usage examples
  - [ ] `examples/automation.rs` - AI agent usage
  - [ ] `examples/json_parsing.rs` - Parsing JSON output
  - [ ] `examples/batch_operations.rs` - Bulk task creation
- [ ] Generate API documentation
  - [ ] `cargo doc --open`

**Deliverables:**
- User-facing documentation is complete
- Examples demonstrate common use cases
- API docs are generated and readable

---

### Phase 15: Polish & Release
**Goal:** Final refinements and first release

- [ ] Performance optimization
  - [ ] Profile with cargo flamegraph
  - [ ] Optimize hot paths
  - [ ] Reduce binary size
- [ ] Security audit
  - [ ] Token storage permissions (0600)
  - [ ] Environment variable handling
  - [ ] Input validation
  - [ ] Dependency audit with cargo-audit
- [ ] Cross-platform testing
  - [ ] Test on Linux
  - [ ] Test on macOS
  - [ ] Test on Windows (if supported)
- [ ] Create release artifacts
  - [ ] Build optimized binaries
  - [ ] Create checksums
  - [ ] Package for distribution
- [ ] Set up release process
  - [ ] Semantic versioning
  - [ ] Changelog generation
  - [ ] GitHub releases
- [ ] Consider distribution channels
  - [ ] Cargo.io publishing
  - [ ] Homebrew formula
  - [ ] Binary releases on GitHub

**Deliverables:**
- v1.0.0 release
- Binaries for all platforms
- Published to cargo.io (optional)

---

## Success Criteria

### Functional Requirements
- [ ] All 18 commands from Go version are implemented
- [ ] OAuth authentication works without user interaction after init
- [ ] All commands support `--json` flag
- [ ] Natural language date parsing works for common expressions
- [ ] Configuration persists between sessions
- [ ] Default project context works correctly
- [ ] INBOX project is handled correctly

### Non-Functional Requirements
- [ ] No TUI/interactive components in codebase
- [ ] Binary size <10MB (optimized build)
- [ ] Cold start time <100ms for simple commands
- [ ] JSON output is valid and parseable
- [ ] Error messages include actionable guidance
- [ ] 80%+ code coverage
- [x] Zero clippy warnings in CI
- [ ] Documentation covers all commands

### AI Agent Compatibility
- [ ] JSON output is consistent across all commands
- [ ] All operations can be performed non-interactively
- [ ] Exit codes indicate success/failure
- [ ] Error details in JSON include error codes
- [ ] No unexpected prompts or confirmations
- [ ] Batch operations are efficient (if implemented)

---

## Risk Assessment

### Technical Risks

| Risk | Impact | Mitigation |
|------|--------|------------|
| OAuth flow differs from Go implementation | High | Study Go code carefully, test early |
| Natural language date parsing library gaps | Medium | Choose mature library, test extensively |
| TickTick API undocumented behavior | Medium | Reference Go implementation, add logging |
| Binary size too large | Low | Use release profile optimizations |

### Schedule Risks

| Risk | Impact | Mitigation |
|------|--------|------------|
| Underestimated OAuth complexity | Medium | Allocate extra time in Phase 4 |
| Testing takes longer than planned | Medium | Start integration tests early |
| Natural language parsing is complex | Low | Can fallback to ISO dates only |

---

## Future Enhancements (Out of Scope)

The following features are not part of the initial refactor but could be added later:

- [ ] Offline mode with local cache
- [ ] Batch operations (bulk create/update)
- [ ] Custom output formats (CSV, Markdown)
- [ ] Task templates
- [ ] Recurring task management UI
- [ ] Subtask create/update/delete operations
- [ ] Advanced filtering (complex queries)
- [ ] Shell completion scripts
- [ ] Man pages
- [ ] Plugin system for extensions

---

## Appendix

### Command Reference Mapping

| Go Command | Rust Command | Notes |
|------------|--------------|-------|
| `tickli init` | `tickrs init` | Same OAuth flow |
| `tickli reset` | `tickrs reset` | Same behavior |
| `tickli version` | `tickrs version` | Add --json support |
| `tickli project list` | `tickrs project list` | Add --json |
| `tickli project show <id>` | `tickrs project show <id>` | Add --json |
| `tickli project use <name>` | `tickrs project use <name>` | Add --json |
| `tickli project create` | `tickrs project create` | Same flags + --json |
| `tickli project update` | `tickrs project update` | Same flags + --json |
| `tickli project delete` | `tickrs project delete` | Add --force, --json |
| `tickli task list` | `tickrs task list` | Remove interactive selector |
| `tickli task show <id>` | `tickrs task show <id>` | Add --json |
| `tickli task create` | `tickrs task create` | Remove -i flag |
| `tickli task update` | `tickrs task update` | Same flags + --json |
| `tickli task delete` | `tickrs task delete` | Add --force, --json |
| `tickli task complete` | `tickrs task complete` | Add --json |
| `tickli task uncomplete` | `tickrs task uncomplete` | Add --json |
| `tickli subtask list` | `tickrs subtask list` | Add --json |

### Environment Variables

```bash
# Required for OAuth
TICKTICK_CLIENT_ID=your_client_id
TICKTICK_CLIENT_SECRET=your_client_secret

# Optional
RUST_LOG=info  # Logging level
```

### Configuration File Format

**Location:** `~/.config/tickrs/config.toml`

```toml
# Default project to use for commands
default_project_id = "abc123"

# Default color for new projects
default_project_color = "#FF1111"
```

### Token Storage

**Location:** `~/.local/share/tickrs/token`

Plain text file containing the OAuth access token with 0600 permissions (owner read/write only).

---

## Change Log

| Version | Date | Changes |
|---------|------|---------|
| 1.0 | 2026-01-13 | Initial PRD based on tickli analysis |

