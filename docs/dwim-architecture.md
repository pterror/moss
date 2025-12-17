# DWIM Architecture

This document describes the internals of the DWIM (Do What I Mean) module for maintainers.

## Overview

`moss.dwim` provides semantic tool routing - matching user intent to the right tool even when the request is imprecise. It's designed for LLM robustness, handling typos, aliases, and natural language descriptions.

## Components

### 1. TF-IDF Index (`TFIDFIndex`)

Pure Python implementation of TF-IDF (Term Frequency-Inverse Document Frequency) for semantic similarity.

**Location**: `src/moss/dwim.py:87-126`

```python
class TFIDFIndex:
    documents: list[str]      # Original text
    doc_tokens: list[list[str]]  # Tokenized
    idf: dict[str, float]     # Inverse document frequency
    doc_vectors: list[dict[str, float]]  # TF-IDF vectors
```

**Key functions**:
- `tokenize(text)` - Splits on word boundaries (`\b\w+\b`)
- `compute_tf(tokens)` - Term frequency (count / total)
- `compute_idf(documents)` - IDF with smoothing: `log((N+1)/(df+1)) + 1`
- `cosine_similarity(vec1, vec2)` - Sparse vector cosine similarity

**Design decisions**:
- No external dependencies (pure Python)
- Sparse vectors as dicts for memory efficiency
- Recomputes IDF on each add (small corpus, negligible cost)

### 2. Tool Registry (`TOOL_REGISTRY`)

Static registry mapping tool names to metadata.

**Location**: `src/moss/dwim.py:134-233`

```python
TOOL_REGISTRY: dict[str, ToolInfo] = {
    "skeleton": ToolInfo(
        name="skeleton",
        description="Extract code structure showing classes, functions, methods",
        keywords=["structure", "outline", "hierarchy", ...],
        parameters=["path", "pattern"],
    ),
    ...
}
```

**Adding a new tool**:
1. Add entry to `TOOL_REGISTRY` with description, keywords, and parameters
2. Add any semantic aliases to `TOOL_ALIASES`
3. The router automatically indexes the new tool

### 3. Alias Maps

**Tool aliases** (`TOOL_ALIASES`): Alternative names for tools
```python
"structure" → "skeleton"
"imports" → "deps"
"search" → "query"
```

**Parameter aliases** (`PARAM_ALIASES`): Alternative parameter names
```python
"file" → "path"
"glob" → "pattern"
"base" → "inherits"
```

### 4. Tool Router (`ToolRouter`)

Main routing class that combines multiple signals.

**Location**: `src/moss/dwim.py:441-529`

**Signal weights** (in `analyze_intent`):
- TF-IDF cosine similarity: 40%
- Keyword matching: 35%
- Description similarity: 15%
- Name similarity: 10%

**Why these weights**:
- TF-IDF captures semantic meaning across the full description
- Keywords are curated for domain knowledge
- Description/name similarity handles exact matches and typos

### 5. Confidence Thresholds

```python
AUTO_CORRECT_THRESHOLD = 0.85  # Auto-correct if >= 0.85
SUGGEST_THRESHOLD = 0.50       # Suggest if >= 0.50
```

**Behavior**:
- `>= 0.85`: Return resolved tool with message "(auto-corrected)"
- `0.50 - 0.84`: Return suggestion "Did you mean X?"
- `< 0.50`: Return no match

## Data Flow

```
User Input
    │
    ▼
┌───────────────────┐
│ Exact Match?      │──yes──► Return tool (confidence=1.0)
└───────────────────┘
    │ no
    ▼
┌───────────────────┐
│ Alias Match?      │──yes──► Return canonical tool (confidence=1.0)
└───────────────────┘
    │ no
    ▼
┌───────────────────┐
│ Fuzzy Match       │
│ (TF-IDF + keywords│
│  + string sim)    │
└───────────────────┘
    │
    ▼
┌───────────────────┐
│ Confidence Check  │
│ >= 0.85: auto     │
│ >= 0.50: suggest  │
│ < 0.50: no match  │
└───────────────────┘
```

## MCP Integration

Three MCP tools expose DWIM functionality:

| Tool | Function | Use Case |
|------|----------|----------|
| `analyze_intent` | `dwim.analyze_intent()` | "What tool should I use for X?" |
| `resolve_tool` | `dwim.resolve_tool()` | "Did I spell this tool name right?" |
| `list_capabilities` | `dwim.list_tools()` | "What tools exist?" |

**Location**: `src/moss/mcp_server.py:611-688`

## Testing

Tests in `tests/test_dwim.py` cover:
- TF-IDF correctness (tokenization, cosine similarity, index queries)
- Fuzzy matching (exact, similar, different strings)
- Tool resolution (exact, alias, typo, unknown)
- Parameter normalization
- Router behavior (various intent queries)
- Edge cases (empty, long, special characters)

## Future Improvements

1. **Embedding support**: Replace TF-IDF with neural embeddings for better semantic matching
2. **Learning from usage**: Track which tools users actually want and adjust weights
3. **Context-aware routing**: Consider previous tools used in session
4. **Parameter inference**: Suggest parameter values based on context

## Performance

- Index build: O(n * m) where n=tools, m=avg description length
- Query: O(n) comparisons per query
- Memory: ~10KB for current tool set

For the current ~7 tools, routing is effectively instant (<1ms).
