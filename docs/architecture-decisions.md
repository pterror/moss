# Architecture Decisions

This document records key architectural decisions and their rationale.

## Language Choice: Rust + Python Hybrid

**Decision**: Moss uses Rust for plumbing, Python for interface.

See `docs/rust-python-boundary.md` for the full decision framework.

### Division of Labor

**Rust (moss-cli, moss-core, moss-languages):**
- Tree-sitter parsing for 17+ languages
- SQLite-backed symbol/call graph index
- Fast queries (callers, callees, deps, complexity)
- Structural operations (view, edit, analyze core)
- Daemon for background indexing

**Python (packages/moss-*):**
- LLM integration (edit synthesis, agents)
- Orchestration (workflows, state machines)
- User interfaces (CLI wrapper, TUI, MCP, LSP)
- Rich analysis (patterns, clones, test coverage)
- Plugins (generators, validators, view providers)

### Why This Split?

**Rust for deterministic, performance-critical, syntax-aware operations.** Python's GIL and interpreter overhead made indexing 100k+ file repos slow. Rust with rayon gives true parallelism.

**Python for LLM orchestration and rapid iteration.** The AI ecosystem is Python-first. Prototyping new agent behaviors is faster in Python.

### The Shim Pattern

Python calls Rust via subprocess:
```
Python CLI → rust_shim.passthrough() → Rust binary → JSON output
```

Rules:
- Rust commands always support `--json` for machine consumption
- Passthrough commands bypass Python entirely for speed
- Python gracefully degrades if Rust binary unavailable

## Local Neural Network Memory Budget

**Problem**: Local models for summarization/embeddings can be memory-hungry.

### Model Size Reference

| Model | Params | FP32 | FP16 |
|-------|--------|------|------|
| all-MiniLM-L6-v2 | 33M | 130MB | 65MB |
| distilbart-cnn | 139M | 560MB | 280MB |
| T5-small | 60M | 240MB | 120MB |
| T5-base | 220M | 880MB | 440MB |
| T5-large | 770M | 3GB | 1.5GB |
| T5-3B | 3B | 12GB | 6GB |

### Recommendations

1. **Default to smallest viable model**: T5-small or distilbart-cnn for summarization
2. **Make model configurable**: Users with more RAM can opt for larger models
3. **Lazy loading**: Don't load models until first use
4. **Graceful degradation**: If model loading fails (OOM), fall back to extractive methods or skip
5. **Consider quantization**: INT8 reduces memory ~4x with minimal quality loss

### Pre-summarization Strategy

For web fetching and document processing, use a tiered approach:

1. **Zero-cost extraction** (always): title, headings, OpenGraph metadata
2. **Extractive** (cheap): TextRank, TF-IDF sentence ranking - no NN needed
3. **Small NN** (optional): all-MiniLM for embeddings, distilbart for abstractive
4. **LLM** (expensive): Only when extractive/small NN insufficient

Configuration in `.moss/config.yaml`:
```yaml
summarization:
  model: "distilbart-cnn"  # or "t5-small", "extractive-only", "none"
  max_memory_mb: 500       # Skip NN if would exceed
  fallback: "extractive"   # What to do if model unavailable
```

## Future Considerations

### When to Consider Rewriting in Another Language

**Don't rewrite unless**:
- Profiling shows Python is the bottleneck (not I/O, not subprocesses)
- The bottleneck can't be solved with PyO3/Cython
- The rewrite scope is bounded (single hot module, not entire codebase)

**Likely candidates for Rust extraction**:
- AST diffing algorithms
- Large-scale structural matching
- Real-time incremental parsing

### Hybrid Architecture Pattern

If performance becomes critical, consider the ruff/uv pattern:
- Core algorithms in Rust
- Python wrapper for API/CLI
- Best of both worlds: performance + ecosystem
