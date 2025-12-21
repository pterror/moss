# Moss Roadmap

See `CHANGELOG.md` for completed work. See `docs/` for design docs.

## Next Up

- [ ] Tokens per symbol path in telemetry (leverage codebase tree)
- [ ] Real-time telemetry mode (`moss telemetry --watch`)
- [ ] Add Gemini CLI log parser to plugin system

## Active Backlog

**When done or stuck, do this:**
- [ ] **Dogfooding & Agentic Loop Iteration**:
  1. Run a moss agentic loop on part of the codebase:
     - `moss agent "task description"` - **Primary**: DWIM-driven agent with parse→execute loop
     - `moss workflow run vanilla --file <file> --arg "task=..."` - Single-shot prompt test
     - `moss workflow run validate-fix --file <file>` - validate and fix errors
     - `moss loop run simple --file <file>` - simple loop
  2. Evaluate results, log to `logs/dogfood-YYYY-MM-DD.md` (rotate when too long)
  3. Plan improvements to test (loop behavior, infra, prompts)
  4. Repeat
  - Keys: see `.env.example` for ANTHROPIC_API_KEY, OPENAI_API_KEY, GEMINI_API_KEY

**Large:**
- [ ] **CLI Cleanup & Consolidation**:
  - Many commands need reconsidering: generalize, redesign, merge, or remove
  - `dwim` - may no longer be necessary with 3 primitives
  - `workflow` vs `loop` - redundant? consolidate?
  - `patterns` - slow, consider Rust port
  - `git-hotspots` - slow (may be unavoidable)
  - `roadmap` - missing --compact mode
  - Missing --compact on various commands
  - See also: Phase 3 in "Codebase Tree Consolidation" below
- [ ] **Comprehensive Telemetry & Analysis**: (In Progress - see `docs/telemetry.md`)
  - [x] `moss telemetry` CLI with aggregate analysis
  - [x] HTML dashboard output
  - [x] Plugin architecture for log formats (LogParser protocol)
  - [ ] Tokens per function/file/module
- [ ] Memory system - layered memory for cross-session learning (see `docs/memory-system.md`)

## Future Work

### Agent Research & Optimization
- [ ] **Conversational Loop Pattern**: Add optional conversation-accumulating mode to DWIMLoop
  - For evals: measure context utilization vs hierarchical approach
  - Session-wide RAG: retrieve from full session history, not just TaskTree
  - Compare: hierarchical (current) vs conversational vs hybrid approaches
- [ ] **YOLO Mode Evaluation**: Evaluate if a "YOLO mode" aligns with Moss architectural principles
- [ ] **'Diffusion-like' methods for large-scale refactors**:
  - Parallelize implementation of components based on high-level contracts
- [ ] **Fine-tuned Tiny Models**: Extreme optimization with models like 100M RWKV

### Codebase Tree Consolidation (see `docs/codebase-tree.md`)

**Phase 1: Python CLI delegates to Rust** (complete)
- [x] `skeleton` → Rust `skeleton`
- [x] `summarize` → Rust `summarize`
- [x] `expand` → Rust `expand`
- [x] `callers` → Rust `callers`
- [x] `callees` → Rust `callees`
- [x] `anchors` → Rust `anchors`
- [x] `tree` → Rust `tree`
- `query` - Python-only (rich filtering Rust lacks, no delegation needed)

**Phase 2: Unified tree model** (partially complete)
- [x] Uniform node addressing with `/`: `src/main.py/Foo/bar`
  - Filesystem is source of truth for file vs directory boundary
  - Accept multiple separators: `/`, `::`, `:`, `#`
  - Normalize all to canonical `/` form internally
- [x] Depth-based expansion: `--depth 1` (default), `--depth 2`, `--all`
- [x] `view [path]` - see node (skeleton, source, tree) with `--deps`
- [x] `view` with filters: `--type`, `--calls`, `--called-by` (find unified into view)
- [x] `edit <path>` - modify node with `--delete`, `--replace`, `--before`, `--after`, `--prepend`, `--append`, `--move-*`, `--swap`
- [x] `analyze [path]` - compute properties with `--health`, `--complexity`, `--security`

**Phase 3: Simplify tool interface** (complete)
- [x] Remove DWIM embedding system (fastembed/bge-small-en dependency removed)
- [x] Simple tool resolution: exact match + basic typo correction for 3 names
- [x] Keep path fuzzy resolution (already in Rust): `view dwim` → `src/moss/dwim.py`
- [x] Consolidate MossAPI: 30 sub-APIs → 3 primitive APIs matching CLI/MCP

### Distribution & Installation
- [ ] Auto-updates: check for new versions, prompt user
- [ ] Portable installation: single binary or minimal deps
- [ ] Pre-built binaries for common platforms (GitHub Actions)

### Reference Resolution (GitHub-level)
- [ ] Full import graph with alias tracking
- [ ] Variable scoping analysis
- [ ] Type inference for method calls
- [ ] Cross-language reference tracking (Python ↔ Rust) (Partially Complete)

## To Consolidate

*From dogfooding session 2025-12-22 (see `logs/dogfood-2025-12-22.md`):*
- [ ] Agent path resolution fails without full paths (`session_analysis.py` → should find `src/moss/session_analysis.py`)
- [ ] Agent stuck in retry loop (3x same intent) - needs fallback strategy
- [x] Workflow verbose mode - show LLM outputs (--verbose flag added)
- [x] Update vanilla prompt to use 3 primitives (view, edit, analyze)
- [x] Update default model in dwim_loop.py (now uses gemini-2.5-flash)
- [x] Update CORE_PRIMITIVES from 4 to 3 (search folded into view)
- [x] More CLI commands delegate to Rust (cfg, complexity, context, deps, grep, health, overview)

## Notes

### Design Principles
See `docs/philosophy.md` for full tenets. Key goals:
- **Generalize, Don't Multiply**: One flexible solution over N specialized ones
- **Three Primitives**: view, edit, analyze (composable, not specialized)
- Minimize LLM usage (structural tools first)
- Maximize useful work per token
- Low barrier to entry, works on messy codebases