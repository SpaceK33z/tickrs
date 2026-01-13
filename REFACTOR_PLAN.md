# Tickli Refactor: Go to Rust

## Project Goal

Refactor the `tickli` CLI tool from Go to Rust, with a focus on making it AI agent-friendly rather than human-interactive.

## Source

Original Go implementation: `../tickli`

## Key Requirements

### 1. Skip TUI Components
- The Text User Interface (TUI) parts should be omitted
- Focus on command-line interface functionality only
- Target audience: AI agents, not interactive human users

### 2. JSON Output Mode
- Add `--json` flag for all commands
- Enable machine-readable output for AI agent consumption
- Should provide structured data that agents can parse reliably

## Target Outcome

A Rust-based CLI tool that:
- Maintains core functionality from the original tickli
- Removes interactive/TUI features
- Provides JSON output modes for programmatic use
- Is optimized for automation and AI agent workflows

## Next Steps

1. Analyze the existing tickli codebase to identify:
   - Core commands and functionality
   - TUI components to exclude
   - Data structures to preserve

2. Design Rust architecture:
   - Command structure using clap or similar
   - JSON serialization strategy
   - Core business logic modules

3. Implement incrementally:
   - Set up Rust project structure
   - Port commands one by one
   - Add JSON output for each command
   - Test with AI agent use cases
