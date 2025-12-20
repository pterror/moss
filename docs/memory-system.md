# Memory System Design

Layered memory for agent context, making moss feel less "forgetful" across tasks and sessions.

## Philosophy

- **Minimal by default**: Only vital context in automatic layer
- **Configurable**: Users control what's loaded, what triggers
- **Extensible**: Plugin architecture for custom memory sources
- **Proactive, not reactive**: Smart selection, not compression after the fact

## Three Layers

### 1. Automatic (always loaded)

Prepended to system prompt. Minimal by default.

**Default plugins:**
- `preferences` - Critical user corrections ("don't do X", "always use Y")

**Optional plugins:**
- `conventions` - Project patterns not in CLAUDE.md
- `recent_session` - Brief context from last session

**Design principle:** If it's in CLAUDE.md or .moss/config.toml, don't duplicate in memory.

### 2. Triggered (pattern-activated)

Injected when patterns match current state. Zero cost when no match.

**Default plugins:**
- `episodic` - Similar state to past failure → inject warning
- `semantic` - Learned rule matches → surface it

**Examples:**
- Editing file that caused issues before → "Note: last edit to X broke tests"
- Pattern matches known anti-pattern → "Consider: Y approach worked better"

### 3. On-Demand (explicit recall)

Agent has `memory.recall(query)` tool. Queries episodic + semantic stores.

**Use cases:**
- "What happened last time I modified auth.py?"
- "How have similar refactoring tasks gone?"
- "What does the user prefer for error handling?"

## Plugin Protocol

```python
from typing import Protocol, Literal
from moss.memory import StateSnapshot

class MemoryPlugin(Protocol):
    """Plugin for memory sources."""

    @property
    def name(self) -> str:
        """Unique plugin identifier."""
        ...

    @property
    def layer(self) -> Literal["automatic", "triggered", "on_demand"]:
        """Which layer this plugin serves."""
        ...

    async def get_context(self, state: StateSnapshot) -> str | None:
        """Return context if relevant, None otherwise.

        For automatic: always returns context (or empty string)
        For triggered: returns context only if pattern matches
        For on_demand: called with query in state.context
        """
        ...

    def configure(self, config: dict) -> None:
        """Apply configuration from .moss/config.toml."""
        ...
```

## Built-in Plugins

### PreferencesMemory

Layer: `automatic`

Surfaces user corrections and explicit preferences extracted from past sessions.

```python
class PreferencesMemory(MemoryPlugin):
    name = "preferences"
    layer = "automatic"

    def __init__(self, store: PreferenceStore):
        self.store = store
        self.max_items = 10

    async def get_context(self, state: StateSnapshot) -> str:
        prefs = self.store.get_relevant(state.context, limit=self.max_items)
        if not prefs:
            return ""
        return "User preferences:\n" + "\n".join(f"- {p}" for p in prefs)
```

### EpisodicMemory

Layer: `triggered`

Finds similar past episodes and surfaces relevant ones.

```python
class EpisodicMemory(MemoryPlugin):
    name = "episodic"
    layer = "triggered"

    def __init__(self, store: EpisodicStore):
        self.store = store
        self.similarity_threshold = 0.7

    async def get_context(self, state: StateSnapshot) -> str | None:
        similar = await self.store.find_similar(state, limit=3)
        failures = [e for e in similar if e.outcome == Outcome.FAILURE]

        if not failures:
            return None

        warnings = []
        for ep in failures:
            warnings.append(f"- {ep.action.description}: {ep.error_message}")

        return "Past issues with similar context:\n" + "\n".join(warnings)
```

### SemanticMemory

Layer: `triggered`

Matches learned rules against current context.

```python
class SemanticMemory(MemoryPlugin):
    name = "semantic"
    layer = "triggered"

    def __init__(self, store: SemanticStore):
        self.store = store
        self.min_confidence = 0.7

    async def get_context(self, state: StateSnapshot) -> str | None:
        rules = self.store.find_matching_rules(
            state.context,
            min_confidence=self.min_confidence
        )

        if not rules:
            return None

        return "Learned patterns:\n" + "\n".join(
            f"- {r.action} (confidence: {r.confidence:.0%})"
            for r in rules
        )
```

## Configuration

```toml
# .moss/config.toml

[memory]
# Which plugins to load per layer
automatic = ["preferences"]           # minimal default
triggered = ["episodic", "semantic"]  # pattern-based
on_demand = true                      # expose recall() tool

# Plugin-specific config
[memory.preferences]
max_items = 10

[memory.episodic]
similarity_threshold = 0.7
max_warnings = 3

[memory.semantic]
min_confidence = 0.7
```

## Integration Points

### Agent Loop

```python
class LLMToolExecutor:
    def __init__(self, ..., memory: MemoryLayer | None = None):
        self.memory = memory or MemoryLayer.default()

    async def _call_litellm(self, prompt: str) -> tuple[str, int, int]:
        # Inject automatic memory into system prompt
        system = self.config.system_prompt
        if self.memory:
            auto_context = await self.memory.get_automatic()
            if auto_context:
                system = f"{system}\n\n{auto_context}"

        messages = [
            {"role": "system", "content": system},
            {"role": "user", "content": prompt}
        ]
        ...
```

### Before Risky Actions

```python
async def _execute_step(self, step: LoopStep, context: LoopContext) -> StepResult:
    # Check triggered memory before execution
    if self.memory:
        state = StateSnapshot.create(
            files=context.get("files", []),
            context=step.description
        )
        warnings = await self.memory.check_triggers(state)
        if warnings:
            # Inject warnings into step context
            context = context.with_warnings(warnings)

    return await self._run_step(step, context)
```

### As a Tool

```python
# Exposed to agent as memory.recall
async def recall(query: str) -> str:
    """Query memory for relevant past experiences."""
    results = await memory.recall(query)
    return results or "No relevant memories found."
```

## Recording Episodes

Memory needs to be fed. Recording happens:

1. **After tool execution** - outcome of each step
2. **After user correction** - explicit preference
3. **After task completion** - overall success/failure

```python
async def record_outcome(
    state: StateSnapshot,
    action: Action,
    outcome: Outcome,
    error: str | None = None
) -> None:
    episode = Episode.create(state, action, outcome, error_message=error)
    await episodic_store.store(episode)

    # Trigger pattern analysis periodically
    if episodic_store.count() % 100 == 0:
        new_rules = await pattern_matcher.analyze_failures()
        for rule in new_rules:
            semantic_store.add_rule(rule)
```

## Future Extensions

- **Cross-project memory**: Share learnings across repos (opt-in)
- **Memory decay**: Older memories fade, recent ones stronger
- **Contradiction resolution**: When memories conflict, surface for user decision
- **Memory export/import**: Backup, share team conventions
