# Moss Implementation Checklist

## Phase 0: Foundation
- [ ] Project structure (src/, tests/, pyproject.toml)
- [ ] Linting setup (ruff config, pre-commit hook)
- [ ] Basic test harness (pytest)

## Phase 1: Core Primitives
- [ ] Event Bus (pub/sub with typed events)
- [ ] Shadow Git wrapper (atomic commits, rollback, branch management)
- [ ] Handle system (lazy references to files/artifacts)

## Phase 2: Context Engine
- [ ] View Provider protocol (abstract base)
- [ ] Skeleton provider (AST-based, Tree-sitter)
- [ ] Dependency Graph provider
- [ ] View compilation pipeline

## Phase 3: Structural Editing
- [ ] Anchor resolution (fuzzy AST matching)
- [ ] Patch application (AST-aware edits)
- [ ] Fallback to text-based editing for broken AST

## Phase 4: Validation Loop
- [ ] Validator protocol
- [ ] Built-in validators (syntax, ruff, pytest)
- [ ] Silent loop orchestration (draft → validate → fix → commit)
- [ ] Velocity monitoring (detect oscillation/stalls)

## Phase 5: Policy & Safety
- [ ] Policy engine (intercept tool calls)
- [ ] Quarantine mode (lock broken files)
- [ ] Velocity checks

## Phase 6: Memory Layer
- [ ] Episodic store (state, action, outcome logging)
- [ ] Vector indexing for episode retrieval
- [ ] Semantic rule extraction (offline pattern matcher)

## Phase 7: Multi-Agent
- [ ] Ticket protocol (task, handles, constraints)
- [ ] Worker lifecycle (spawn, execute, die)
- [ ] Manager/merge conflict resolution

## Phase 8: Configuration
- [ ] Executable config (TypeScript/Python DSL)
- [ ] Distro system (composable presets)

## Phase 9: API Surface
- [ ] Headless HTTP API
- [ ] SSE/WebSocket streaming
- [ ] Checkpoint approval endpoints
