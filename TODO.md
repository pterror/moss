# Moss Roadmap

See `CHANGELOG.md` for completed work. See `docs/` for design docs.

## Next Up

- [ ] **Recursive Workflow Learning**: Propose new workflows based on recurrent session patterns
- [ ] **Memory & Resource Metrics**: Show context and RAM usage (with breakdown) for every command
- [ ] **Local Model Constrained Inference**: Implement GBNF (GGML BNF) for structured output
- [ ] **TUI Syntax Highlighting**: High-quality code highlighting in file previews
- [ ] **Adaptive Model Rotation**: Dynamically switch LLM providers based on task latency
- [ ] **Agentic Prompt Versioning**: Implement loop that manages and compares prompt evolutions

## Recently Completed (Dec 2025)

### Agent & Core Infrastructure
- **Sandbox Scoping**: Task-level workspace restriction with parent inheritance and automatic enforcement.
- **Workflow Loader Abstraction**: Extracted `WorkflowLoader` protocol and `TOMLWorkflowLoader` with registry.
- **Vanilla Workflow**: Minimal baseline agent loop refactored into a data-driven workflow.
- **TelemetryAPI**: Unified analysis of multi-session token usage, tool patterns, and hotspots.
- **Adaptive Workspace Scoping**: Dynamic sandbox control with `shrink_to_fit` and `expand_to_include`.
- **RefCheck**: Cross-language reference tracking for Rust/Cargo with deduplication.

### Adaptive Loop Capabilities
- **Adaptive Context Control**: Dynamic result preview limits based on task type (Read vs Write).
- **Adaptive Context Pruning**: Heuristic and LLM-driven importance scoring for intelligent elision.
- **Adaptive Loop Depth**: Dynamic `max_steps` adjustment in `AgentLoopRunner` based on progress.
- **Dynamic Turn Budgeting**: Per-turn token scaling based on estimated task complexity.
- **Adaptive Model Selection**: Task-specific model routing (e.g., using different models for analysis vs generation).
- **LLM Benchmarking Harness**: Automated cross-model evaluation with markdown report generation.

### Recursive Improvement Loops
- **Recursive Policy Learning**: Automated distillation of safety rules from successful histories.
- **Agentic Prompt Evolution**: Automated system prompt refinement based on session feedback.
- **Adaptive Loop Strategy Refinement**: History-based switching between DWIM and Structured loops.
- **Agentic Tool Discovery**: Automated search and configuration of new MCP tools.
- **Agentic Workflow Synthesis**: Automatic creation of new workflows from telemetry patterns.
- **Recursive Self-Optimization**: Tuning of structural heuristics based on session outcomes.

### TUI & User Experience
- **Extensible Agent Modes**: Plugin-based TUI mode system (PLAN, READ, WRITE, DIFF, SESSION, BRANCH, SWARM, COMMIT).
- **TUI Git Dashboard**: Integrated view for branches, commits, hunks, and diffs with surgical rollback.
- **TUI Session Resume**: Visual session history with one-click resumption and state recovery.
- **Cross-file Symbol Jump**: Clickable references in TUI for quick navigation between files.
- **Symbol Hover Info**: Metadata tooltips (skeletons, summaries) in the ProjectTree.
- **TUI Exit Refinement**: Double `Ctrl+C` exit to avoid clipboard conflicts.
- **Docs Styling**: Modern glassmorphism and rounded borders at `docs/stylesheets/custom.css`.

### Safety & Verification
- **LLM Reliability Guardrails**: 'Critic-first' execution for high-risk operations.
- **Heuristic Error Localization**: Trace-based bug identification from test failures.
- **Mistake Detection**: Dedicated critic steps for turn-level logic analysis.
- **Verification Loops & Heuristics**: Formalized structural guardrails before full validation.
- **Shadow Git Access**: First-class LLM access to diffs, hunks, multi-commits, and smart merging.
- **User Feedback Story**: Agent inbox for mid-task corrections.
- **Editing Tools**: `EditAPI` for direct file manipulation (write, replace, insert).

## Active Backlog

- Workflow argument passing improvement
- [ ] **Symbol Hover Info**: (TUI) Show signatures/docstrings on hover (Expanded)
- [ ] **Context Elision Heuristics**: (Core) Prune large files while preserving anchors (Expanded)
- [ ] **Shadow Git Branching**: (Git) Support for multiple concurrent experiment branches (Expanded)

**Large:**
- [ ] **Comprehensive Telemetry & Analysis**: (Partially Complete - see TelemetryAPI)
  - Track all token usage, patterns, and codebase access patterns by default
  - Store maximal metadata for every session
  - Built-in high-quality analysis tools (CLI & visual)
- [ ] Memory system - layered memory for cross-session learning (see `docs/memory-system.md`)

## Future Work

### Agent Research & Optimization
- [ ] **LLM Editing Performance Comparison**:
  - Investigate Gemini 3 Flash and Gemini 3 Pro issues with invalid code edits
  - Compare with Claude Code and Opus to identify architectural differences
  - Evaluate if specialized prompting or different edit formats (e.g. diffs) help
- [ ] **YOLO Mode Evaluation**: Evaluate if a "YOLO mode" aligns with Moss architectural principles
- [ ] **Memory Usage Optimization**: Ensure Moss keeps RAM usage extremely low, even for large codebases
- [ ] **'Diffusion-like' methods for large-scale refactors**:
  - Generate contracts/signatures at high levels first
  - Parallelize implementation of components
  - Explore reconciliation strategies for independent components
- [ ] **Small/Local Model Brute Force**: Explore using smaller, faster local models with higher iteration/voting counts
- [ ] **Fine-tuned Tiny Models**:
  - Explore extreme optimization with models like 100M RWKV
  - Benchmark model size vs. reliability
  - High-frontier LLM generated tests for tiny model validation

### Codebase Tree Consolidation (see `docs/codebase-tree.md`)

**Phase 1: Python CLI delegates to Rust (remove Python implementations)**
- [ ] `skeleton` → delegate to Rust `view`
- [ ] `summarize` → delegate to Rust `view`
- [ ] `anchors` → delegate to Rust `search` with type filter
- [ ] `query` → delegate to Rust `search`
- [ ] `tree` → delegate to Rust `view` (directory-level)

**Phase 2: Unified tree model**
- [ ] Merge filesystem + AST into single tree data structure
- [ ] Implement zoom levels (directory → file → class → method → params)
- [ ] Consistent "context + node + children" view format

**Phase 3: DWIM integration**
- [ ] Natural language → tree operation mapping
- [ ] "what's in X" → view, "show me Y" → view, "full code of Z" → expand

### Reference Resolution (GitHub-level)
- [ ] Full import graph with alias tracking (`from x import y as z`)
- [ ] Variable scoping analysis (what does `x` refer to in context?)
- [ ] Type inference for method calls (`foo.bar()` where `foo: Foo`)
- [ ] Cross-language reference tracking (Python ↔ Rust)

## Notes

### Key Findings
- **86.9% token reduction** using skeleton vs full file (dwim.py: 3,890 vs 29,748 chars)
- **12x output token reduction** with terse prompts (1421 → 112 tokens)
- **90.2% token savings** in composable loops E2E tests
- **93% token reduction** in tool definitions using compact encoding (8K → 558 tokens)

### Design Principles
See `docs/philosophy.md` for full tenets. Key goals:
- Minimize LLM usage (structural tools first)
- Maximize useful work per token
- Low barrier to entry, works on messy codebases
- **Heuristic Guardrails**: Mitigate LLM unreliability with verification loops and deterministic rules
