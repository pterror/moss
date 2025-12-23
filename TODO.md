# Moss Roadmap

See `CHANGELOG.md` for completed work. See `docs/` for design docs. See `docs/dogfooding.md` for testing notes.

## Next Up

**TUI Redesign** (see `docs/tui-design.md`)

Phase 1 - Unified task model:
- Unify session/workflow/swarm into "task" concept
- Each task gets shadow branch `shadow/task-{id}`
- Task tree shows all work (active, paused, completed)
- Tasks are persistent, shadow branches never deleted

Phase 2 - Simplify panels:
- Remove redundant modes (PLAN, READ, WRITE, DIFF, SESSION, BRANCH, SWARM, COMMIT)
- Three panels: Code, Analysis, Tasks
- Analysis has sub-views: Complexity, Security, Scopes, Imports
- Diff is accessed through task's changes, not separate mode

Phase 3 - No commands:
- Remove command input
- Direct manipulation only (navigate + contextual actions)
- Footer shows available actions for current context
- `e` on file opens edit task input (the one text input we need)

Immediate fixes (while redesigning):
- Gracefully handle missing Shadow Git in current DIFF mode
- Mode names to title case (temporary until modes removed)

## Implementation Notes

**Self-update (`moss update`):**
- GITHUB_REPO constant in main.rs:4004 is set to "pterror/moss"
- Uses custom SHA256 implementation (no external crypto dep) in main.rs:4220-4310
- Expects GitHub release with SHA256SUMS.txt containing checksums
- Binary replacement creates temp file then renames (atomic on Unix)

## Backlog

**Reference Resolution (partial):**
- Cross-language tracking (Python ↔ Rust) - see `docs/rust-python-boundary.md` for design

**Deferred:**
- Python edit separate targeting (LLM-based, intentionally different)
- Remaining docs: prior-art.md, hybrid-loops.md (lower priority)

**Fisheye for Other Languages:**
- Go (import resolution from go.mod)
- Java (package/class resolution)
- C/C++ (#include resolution)
- Ruby (require resolution)

**Call Graph:**
- Missing language support: Scala, Vue (no tree-sitter grammars yet)
- "(no ext)" files high count in some repos - add binary detection
- Wire FunctionComplexity.short_name() into complexity output
- Complete daemon integration (FileIndex API methods currently unused)

**Session Analysis:**
- Correction pattern detection: flag "You're right", "Good point", "Ah yes", etc.
- Could be a `moss analyze-session` tool or part of telemetry
- Use detected corrections to identify friction points

**Editor Integration:**
- LSP refactor actions (rename symbol across files via language server)

**Memory System:**
- Layered memory for cross-session learning (see `docs/memory-system.md`)

**Agent TUI:**
- Terminal output sanitization: reset terminal state after nested command output

**Agent Research:**
- Conversational loop pattern (vs hierarchical)
- YOLO mode evaluation
- Diffusion-like parallel refactors
- Fine-tuned tiny models (100M RWKV)
- Analyze ampcode research notes (ampcode.com/research) for deeper patterns

## Notes

### Design Principles
See `docs/philosophy.md`. Key goals:
- **Generalize, Don't Multiply**: One flexible solution over N specialized ones
- **Three Primitives**: view, edit, analyze (composable, not specialized)
- Minimize LLM usage (structural tools first)
- Maximize useful work per token

### API Keys
See `.env.example` for ANTHROPIC_API_KEY, OPENAI_API_KEY, GEMINI_API_KEY

## When Ready

**First Release**
- Create first GitHub release to test distribution pipeline:
  ```bash
  git tag v0.1.0
  git push --tags
  ```
- Verify cross-platform builds succeed in GitHub Actions
- Test `moss update` against the real release
