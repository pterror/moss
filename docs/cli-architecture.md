# CLI Architecture

This document describes the Moss CLI architecture.

## Two CLIs

Moss has two CLI implementations:

1. **Rust CLI** (`crates/moss-cli/`) - Primary implementation
   - Fast, parallel operations
   - All core commands: view, edit, analyze, tree, callers, callees, deps, etc.
   - 29 command modules in `commands/` directory

2. **Python CLI** (`packages/moss-cli/`) - Thin wrapper
   - Passes through to Rust for most commands
   - Handles LLM-related commands (agent, workflow)
   - Orchestration and TUI

## Command Routing

```
User runs `moss <command>`
    ↓
Python CLI entry point
    ↓
Is it a passthrough command? (view, edit, analyze, tree, etc.)
    ├── YES → rust_shim.passthrough() → Rust binary
    └── NO → Python implementation (agent, workflow, tui)
```

Passthrough commands bypass Python argparse entirely for speed.

## Rust CLI Structure

**Location**: `crates/moss-cli/src/`

```
main.rs           # ~800 lines, argument parsing
commands/         # 29 modules, one per command
  ├── view_cmd.rs
  ├── edit.rs
  ├── analyze_cmd.rs
  ├── callers.rs
  └── ...
```

**Key conventions**:
- All commands support `--json` for machine consumption
- Return 0 for success, non-zero for errors
- Output to stdout, errors to stderr

## Adding Commands

**Rust commands** (preferred for performance-critical):
1. Add module in `crates/moss-cli/src/commands/`
2. Wire into main.rs argument parsing
3. Add `--json` output support

**Python commands** (for LLM/orchestration):
1. Add to `packages/moss-cli/src/moss_cli/_main.py`
2. Use rust_shim for Rust operations

## Output Format

All commands support two output modes:
- **Human-readable**: Default, formatted for terminals
- **JSON** (`--json`): Structured data for scripts/tools

## MCP Server

`moss mcp-server` exposes CLI commands as MCP tools.
Uses the same Rust CLI underneath.
