# Moss Roadmap

See `CHANGELOG.md` for completed features (Phases 15-29).

See `~/git/prose/moss/` for full synthesis design documents.

## Future Work

### Interface Generators

Additional interface generators for the library-first architecture:

- [ ] `moss.gen.lsp` - Generate LSP handlers from API
- [ ] `moss.gen.grpc` - Generate gRPC proto + handlers from API
- [ ] `moss-lsp` entry point (requires `[lsp]` extra)
- [ ] Unix socket transport for local high-performance server

### Non-LLM Code Generators

Alternative synthesis approaches that don't rely on LLMs. See `docs/synthesis-generators.md` for details.

#### High Priority
- [ ] `EnumerativeGenerator` - enumerate ASTs, test against examples (Escher/Myth)
- [ ] `ComponentGenerator` - combine library functions bottom-up (SyPet/InSynth)
- [ ] `SMTGenerator` - Z3-based type-guided synthesis (Synquid)

#### Medium Priority
- [ ] `PBEGenerator` - Programming by Example (FlashFill/PROSE)
- [ ] `SketchGenerator` - fill holes in user templates (Sketch/Rosette)
- [ ] `RelationalGenerator` - miniKanren-style logic programming

#### Research/Experimental
- [ ] `GeneticGenerator` - evolutionary search (PushGP)
- [ ] `NeuralGuidedGenerator` - small model guides enumeration (DeepCoder)
- [ ] `BidirectionalStrategy` - λ²-style type+example guided search

### DreamCoder-style Learning

Advanced abstraction discovery:

- [ ] Compression-based abstraction discovery
- [ ] MDL-based abstraction scoring

### Multi-Language Expansion

- [ ] Full TypeScript/JavaScript synthesis support
- [ ] Go and Rust synthesis strategies

### CLI Output Enhancement

Remaining token-efficient output features:

- [ ] `--query EXPR` flag - relaxed DWIM syntax for flexible querying (needs design work)
- [ ] Format strings for custom output templates

### Codebase Analysis Gaps

Tools we have:
- Project health: `overview`, `health`, `metrics`
- Structure: `skeleton`, `summarize`, `deps`
- Dependencies: `external-deps` (vulns, licenses, weight)
- Quality: `check-docs`, `check-todos`, `check-refs`
- Coverage: `coverage` (pytest-cov stats)
- Complexity: `complexity` (cyclomatic per function)
- Git analysis: `git-hotspots` (frequently changed files)

Potential additions:
- [ ] Architecture diagrams from dependency graph
- [ ] `moss patterns` - Detect and analyze architectural patterns:
  - Plugin systems (Protocol + Registry + Entry Points)
  - Factory patterns, strategy patterns, adapter patterns
  - Inconsistent patterns (e.g., some registries use entry points, others don't)
  - Hardcoded implementations that could be plugins
  - Coupling analysis (which modules know about each other)
  - Report: "X uses plugin pattern, Y could benefit from it"
- [ ] `moss weaknesses` / `moss gaps` - Identify architectural weaknesses and gaps:
  - Hardcoded assumptions (e.g., parsing only supports one format)
  - Missing abstractions (e.g., no plugin system where one would help)
  - Tight coupling between components
  - Single points of failure
  - Missing error handling patterns
  - Inconsistent patterns across similar code
  - Technical debt indicators
  - Self-analysis: moss should be able to identify its own architectural gaps
    (eating our own dogfood, providing actionable feedback during development)

### Agent Log Analysis

Manual analysis complete - see `docs/log-analysis.md` for methodology and insights.
Basic automation: `moss analyze-session <path>` parses Claude Code JSONL logs.
Preference extraction: `moss extract-preferences` and `moss diff-preferences` are now implemented (see Phase 31 in CHANGELOG.md).

### Research: SOTA Coding Agents

Investigate and potentially learn from state-of-the-art coding agents:

- [ ] [SWE-agent](https://github.com/swe-agent/swe-agent) - Princeton's autonomous agent for software engineering tasks
- [ ] [GUIRepair](https://sites.google.com/view/guirepair) - GUI-based program repair

### Enterprise Features

- [ ] Team collaboration (shared caches)
- [ ] Role-based access control

---

## Vision: Augmenting the Vibe Coding Loop

Moss should both **replace** and **augment** conventional AI coding assistants like Claude Code, Gemini CLI, and Codex CLI. The goal is not to compete on chat UX, but to provide the structural awareness layer that makes any agentic coding loop more reliable:

- **As a replacement**: `moss run` can orchestrate full tasks with verification loops, shadow git, and checkpoint approval
- **As an augmentation**: Tools like `moss skeleton`, `moss deps`, `moss check-refs`, `moss mutate` can be called by *any* agent to get architectural context before making changes
- **MCP integration**: `moss-mcp` exposes all capabilities to MCP-compatible agents

The key insight: vibe coding works better when the agent understands structure, not just text.

---

## Notes

### Programming as Recursive Abstraction

Core philosophy: programming is essentially recursive abstraction - we build abstractions on top of abstractions. This has implications for Moss's design:

- **Plugin architecture as abstraction**: Each plugin system is an abstraction layer that can be extended
- **Composable primitives**: Small tools compose into larger capabilities
- **Meta-level tooling**: Tools that analyze/generate other tools (e.g., `moss patterns` analyzing plugin usage)
- **Self-describing systems**: Code that can describe its own structure (skeleton, deps, etc.)

**Continuous zoom / arbitrary granularity views**: Traditional tools have fixed levels (file, function, line), but code structure is fractal - you should be able to view it at any detail level continuously:
- Full source → skeleton → signatures → names → nothing
- Codebase → directories → files → symbols → blocks → expressions
- But NOT as fixed levels - as a continuous spectrum where "blocks", "functions", "files" are just convenient labels for points on a continuum
- The structural units (block, function, file, module) are emergent properties, not fundamental categories
- Goal: `moss view <path> --detail=0.3` where detail is a float, not "skeleton" vs "full"

Research directions:
- [ ] Can Moss analyze its own abstraction layers? (`moss abstractions` command)
- [ ] Automatic abstraction discovery (find repeated patterns that could be factored out)
- [ ] Abstraction quality metrics (coupling, cohesion, depth)
- [ ] Tools for refactoring concrete code into abstract plugins
- [ ] Continuous detail views - parameterized by a single "zoom" float rather than discrete modes
- [ ] Semantic compression: what's the minimum representation that preserves X% of the information?

### PyPI Naming Collision

There's an existing `moss` package on PyPI (a data science tool requiring numpy/pandas). Before publishing, we need to either:
- Rename the package (e.g., `moss-tools`, `moss-orchestrate`, `toolmoss`)
- Check if the existing package is abandoned and claim the name
- Use a different registry

### Remote Agent Management

Web interface for monitoring and controlling agents remotely:

- [ ] Agent manager web server (`moss serve --web`)
  - Real-time status dashboard
  - View running tasks and progress
  - Approve/reject checkpoints from mobile
  - Send commands to running agents
  - View session logs and metrics
- [ ] Mobile-friendly responsive UI
- [ ] WebSocket for live updates
- [ ] Authentication for remote access
