# Moss Tools

**CLI**: `moss <command> [options]` — all commands support `--json`

**MCP Server**: `moss mcp-server`

## Core Primitives

Three tools for all codebase operations:

### view

Show tree, file skeleton, or symbol source.

```
moss view [target] [options]
```

- `view` — project tree
- `view src/` — directory contents
- `view file.py` — file skeleton (fuzzy paths OK)
- `view file.py/Class` — symbol source
- `--depth N` — expansion depth
- `--deps` — show dependencies
- `--calls` — show callers
- `--called-by` — show callees

### edit

Structural code modifications.

```
moss edit <target> [options]
```

- `--delete` — remove node
- `--replace "code"` — swap content
- `--before "code"` — insert before
- `--after "code"` — insert after
- `--prepend "code"` — add to start
- `--append "code"` — add to end

### analyze

Health, complexity, and security analysis.

```
moss analyze [target] [options]
```

- `analyze` — full codebase analysis
- `--health` — file counts, line counts, avg complexity
- `--complexity` — cyclomatic complexity per function
- `--security` — vulnerability scanning

## Legacy Commands

These commands still work but are deprecated in favor of the 3 primitives:

- `skeleton` → use `view`
- `anchors` → use `view --type`
- `query` → use `view` with filters
- `deps` → use `view --deps`
- `cfg` → still available for control flow graphs
- `context` → use `view --deps`
- `health` → use `analyze --health`
- `complexity` → use `analyze --complexity`
- `security` → use `analyze --security`

## DWIM Resolution

Tool names are resolved with fuzzy matching:

- `view`, `show`, `skeleton`, `tree` → view primitive
- `edit`, `modify`, `change`, `patch` → edit primitive
- `analyze`, `check`, `health`, `lint` → analyze primitive

Fuzzy path resolution also works: `dwim.py` → `src/moss/dwim.py`
