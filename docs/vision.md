# Moss Vision

## What Moss Is

Moss is **structural code intelligence as a platform**.

It provides tools for understanding, navigating, and modifying code at a structural level (AST, control flow, dependencies) rather than treating code as text. This foundation serves multiple consumers through multiple interfaces, with the same capabilities available everywhere.

## Who It's For

| User | Interface | Use Case |
|------|-----------|----------|
| Developer | CLI, TUI | Understand unfamiliar code, explore structure |
| AI Agent | MCP, Library | Get structured context, make safe modifications |
| IDE | LSP | Code intelligence, navigation, refactoring |
| CI/CD | CLI | Quality gates, validation, analysis |
| Tool Builder | Library | Build custom tools on structural primitives |

Moss is useful alone and powerful with AI. A human can `moss skeleton` to understand a file; an agent can use the same capability to build context for code generation.

## Core Principles

### 1. Structural Awareness Over Text Processing

Code is structure, not strings. Moss parses, analyzes, and operates on:
- **AST** - Abstract syntax trees for precise code understanding
- **CFG** - Control flow graphs for execution analysis
- **Dependencies** - Import/export relationships, call graphs
- **Anchors** - Fuzzy-matched references to code elements

This enables operations that text processing can't: accurate refactoring, structural similarity, semantic navigation.

### 2. Library-First, Surfaces Generated

The Python library (`MossAPI`) is the source of truth. All interfaces are views onto it:

```
                    ┌─────────────┐
                    │   MossAPI   │
                    │  (Library)  │
                    └──────┬──────┘
                           │
        ┌──────┬───────┬───┴───┬───────┬──────┐
        │      │       │       │       │      │
       CLI    TUI     LSP     MCP    HTTP   gRPC
```

Run a generator → interface stays in sync. No manual maintenance of multiple implementations.

### 3. Useful Defaults, Full Configurability

Ship with opinionated defaults that work for the common case. Let users override when they know better.

```toml
# .moss/config.toml - override defaults when needed
[views]
default = ["skeleton", "dependency"]

[validators]
enabled = ["syntax", "ruff", "pytest"]

[synthesis]
strategy = "type-driven"
```

The curation is in configuration, not code. Defaults evolve based on what we learn works.

### 4. Safe Modification

Code changes are risky. Moss provides:
- **Shadow Git** - Atomic commits on isolated branches
- **Checkpoints** - Save/restore points for experimentation
- **Validator Chains** - Syntax, linting, tests before accepting changes
- **Rollback** - Undo when things go wrong

Safe by default, not by heroic caution.

### 5. Everything Is a Plugin

Core capabilities use the same plugin interfaces as extensions:
- View providers (skeleton, CFG, dependencies, custom)
- Validators (syntax, ruff, mypy, custom)
- Synthesis strategies (type-driven, test-driven, custom)
- Linters (ruff, mypy, custom)

No privileged built-ins. If you don't like how something works, replace it.

## The UX Vision

### Natural Exploration

The power should be discoverable through intuitive interaction, not memorized commands.

**Instead of:** "I need to know `moss skeleton` exists, type it, parse output"

**We want:** "I'm looking at code → what calls this? → show me that → zoom out"

### UX Principles

- **No modals** - Everything inline, no popups blocking context
- **No nested menus** - Flat, searchable action lists
- **Actions visible** - Show what's possible, don't hide capabilities
- **Direct manipulation** - Click/select to act, not navigate menus
- **Mouse support** - Full mouse interaction everywhere (especially TUI)
- **Progressive disclosure** - Start simple, reveal depth on demand

### Unified Navigation Model

Same mental model across all interfaces:

1. **Start anywhere** - File, function, class, symbol, or natural language query
2. **Traverse by relationship** - calls → called-by → imports → similar-to
3. **Zoom fluently** - Full source ↔ skeleton ↔ signature ↔ one-liner
4. **Context preserved** - Breadcrumbs, back/forward, history

Whether you're in the CLI REPL, TUI, or IDE - navigation works the same way.

### Example: `moss explore`

```
$ moss explore src/moss/cli.py

src/moss/cli.py (4500 lines, 47 functions, 12 classes)

[Tab: completions]  [/: search]  [?: help]  [q: quit]

> skeleton
... structural overview ...

> calls cmd_lint
cmd_lint calls:
  → setup_output (cli.py:42)
  → get_linter_registry (plugins/linters.py:601)
  → asyncio.run (stdlib)

> callers cmd_lint
cmd_lint is called by:
  ← main via argparse dispatch

> similar
Structurally similar functions:
  → cmd_complexity (same pattern: setup → analyze → output)
  → cmd_coverage (same pattern)

> "how does argument parsing work"
[LLM-assisted] The CLI uses argparse with subparsers...
```

## How We're Different

### vs. Generic Agent + Tools (e.g., mini-swe-agent)

Mini-swe-agent proves bash + good prompts gets surprisingly far (74% on SWE-bench with 100 lines of code).

Moss isn't trying to be a simpler agent. It's a **toolkit** that agents use:
- Structural awareness that bash can't provide
- Multiple view types optimized for different tasks
- Safe modification infrastructure
- Same tools available to humans, not just agents

### vs. Traditional Code Intelligence (e.g., LSP servers)

Traditional code intelligence is IDE-centric and focused on single-file operations.

Moss provides:
- Codebase-level analysis (not just current file)
- AI-optimized views (token-efficient representations)
- Modification capabilities (not just read-only intelligence)
- Multiple interfaces (not just IDE integration)

### vs. Code Search (e.g., Sourcegraph)

Code search excels at finding things across large codebases.

Moss complements this with:
- Structural analysis (not just text search)
- Local-first operation (no server required)
- Modification and synthesis (not read-only)
- Plugin extensibility

## The North Star

A developer (human or AI) working with an unfamiliar codebase should be able to:

1. **Orient quickly** - Understand structure without reading everything
2. **Navigate naturally** - Follow relationships, not grep through files
3. **Modify safely** - Make changes with confidence they can be undone
4. **Discover capabilities** - Find what's possible through the interface itself

The structural intelligence makes this possible. The UX makes it accessible.
