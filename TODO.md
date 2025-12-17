# Moss Roadmap

See `CHANGELOG.md` for completed features (Phases 15-21, 23-24).

See `~/git/prose/moss/` for full synthesis design documents.

## In Progress

### Phase 22: Synthesis Integration

The synthesis **framework scaffolding** is complete, but **code generation and test validation are not implemented**. The framework can decompose problems but cannot generate or validate code.

#### 22a: Core Framework ✅
- [x] Directory structure (`src/moss/synthesis/`)
- [x] Abstract interfaces (`Specification`, `Context`, `Subproblem`)
- [x] `DecompositionStrategy` ABC
- [x] `Composer` ABC (SequentialComposer, FunctionComposer, CodeComposer)
- [x] `StrategyRouter` (TF-IDF-based)
- [x] `SynthesisFramework` engine
- [x] Integration points for shadow git, memory, event bus
- [x] Tests for framework structure

#### 22b: Code Synthesis Domain 🚧
Strategies exist but cannot generate actual code:
- [x] `TypeDrivenDecomposition` - decomposes by type signature
- [x] `TestDrivenDecomposition` - analyzes tests for subproblems
- [x] `PatternBasedDecomposition` - recognizes CRUD/validation patterns
- [ ] **`TestExecutorValidator`** - run pytest/jest to validate code
  - Design: `~/git/prose/moss/code-synthesis-domain.md` lines 43-119
- [ ] **Atomic code generation** - `_solve_atomic()` returns placeholders
  - Need: LLM integration OR template-based generation
- [ ] **Validation retry loop** - compose, validate, fix, repeat

#### 22c: CLI & Integration 🚧
- [x] `moss synthesize` CLI command (shows decomposition)
- [x] `--dry-run` and `--show-decomposition` flags
- [ ] **`moss edit` integration** - fallback for complex tasks
  - Design: `~/git/prose/moss/code-synthesis-domain.md` lines 462-521
- [ ] Synthesis configuration presets (default/research/production)

#### 22d: Optimization & Learning 🚧
- [x] Caching infrastructure
- [x] Parallel subproblem solving (asyncio.gather)
- [x] Scale testing structure
- [ ] Memory-based strategy learning (record_outcome exists, no learning)
- [ ] Performance benchmarks (need working code generation first)

## What's Needed to Make Synthesis Usable

### Option A: LLM Integration (Recommended)
Add Claude/GPT integration for `_solve_atomic()`:
```python
async def _solve_atomic(self, spec, context, validator, state):
    # Call LLM with specification and context
    code = await llm.generate_code(spec, context)

    # Validate with tests
    result = await validator.validate(code)

    # Retry loop if validation fails
    while not result.passed and state.iterations < max_iterations:
        code = await llm.fix_code(code, result.issues)
        result = await validator.validate(code)
        state.iterations += 1

    return code
```

### Option B: Template-Based Generation
For common patterns, use code templates:
- CRUD operations
- Authentication flows
- Data transformations
- API endpoints

### Required Components
1. **TestExecutorValidator** - Actually run tests (subprocess pytest)
2. **Code generation backend** - LLM or templates
3. **Validation retry loop** - Currently a TODO comment

## Backlog

### Future: Multi-Language Expansion
- Full TypeScript/JavaScript synthesis support
- Go and Rust synthesis strategies

### Future: Enterprise Features
- Team collaboration (shared caches)
- Role-based access control
