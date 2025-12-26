# Filtering Design

Cross-command filtering for view, analyze, and future batch operations.

## Current State

| Filter | view | analyze | edit |
|--------|------|---------|------|
| Symbol kind | `-t, --type` | `--kind` | - |
| Types only | `--types-only` | - | - |
| Private | `--include-private` | - | - |
| Category | - | - | - |
| Glob | - | - | - |

**Inconsistency**: view uses `-t/--type`, analyze uses `--kind`. Should unify.

## Current Filters (view)

```
-t, --type <KIND>       Symbol type: class, function, method, etc.
--types-only            Architectural view (classes, structs, enums, interfaces)
--include-private       Show private symbols (normally hidden by convention)
--focus[=MODULE]        Resolve imports inline at signature level
--resolve-imports       Inline specific imported symbol signatures
```

## Proposed Additions

### Exclude/Only Design Options

**Option A: Globs Only (no categories)**
```bash
moss view src/ --exclude="*_test*" --exclude="test_*" --exclude="**/tests/**"
moss view src/ --only="*.rs"
```
- Pro: Simple, no magic, users know globs
- Pro: No DSL creep
- Con: Verbose for common cases (tests = 5+ patterns per language)
- Con: Users must know language-specific test conventions

**Option B: Separate Flags**
```bash
moss view src/ --exclude-category=tests --exclude-pattern="*.gen.go"
moss view src/ --only-category=tests
```
- Pro: Explicit, no ambiguity
- Con: Four flags instead of two
- Con: Verbose

**Option C: Sigil/Prefix**
```bash
moss view src/ --exclude=@tests --exclude="*.gen.go"
moss view src/ --only=@tests --only="*.rs"
```
- Pro: One flag pair, explicit distinction
- Con: DSL creep (what's next, `#regex:` prefix?)
- Con: Sigil choice is arbitrary (@, :, %)

**Option D: Smart Detection**
```bash
moss view src/ --exclude=tests --exclude="*.gen.go"
```
- Pro: Clean syntax
- Con: Magic (contains `*?[` → glob, else category)
- Con: What if category name contains special chars? (unlikely but...)
- Con: Implicit behavior surprises users

**Option E: Categories as Aliases**

Categories are just named glob sets in config, `--exclude` only takes globs:
```toml
# .moss/config.toml or global config
[filter.categories]
tests = ["*_test.*", "test_*.*", "**/tests/**", "**/__tests__/**", "*.spec.*"]
```
```bash
moss view src/ --exclude=@tests          # expands to globs from config
moss view src/ --exclude="*.gen.go"      # literal glob
```
- Pro: User-configurable categories
- Pro: Explicit expansion (@ means "look up in config")
- Pro: No hardcoded language knowledge
- Con: Requires config for categories to work
- Con: Still has a sigil

**Option F: Subcommand for Complex Filtering**
```bash
moss view src/ --exclude="*_test*"       # simple glob only
moss filter tests | moss view src/       # piped filter spec (overdesigned?)
```
- Pro: Simple base case, complex cases are explicit
- Con: Overengineered

---

**Recommendation:** Option A (globs only) or Option E (config-based categories).

Option A is simplest. If we want categories, Option E keeps them user-defined rather than hardcoded, and the `@` sigil is explicit about "this is a lookup, not a literal pattern".

### Symbol Kind Filter (extend existing)

Current `-t/--type` accepts single value. Extend to:

```
-t, --type <KIND,...>   Filter by symbol kinds (comma-separated)
```

Examples:
```bash
moss view file.py -t class,function      # Classes and top-level functions
moss view file.py -t method              # Only methods (inside classes)
```

## Design Decisions

### 1. Filter Precedence

1. `--only` takes precedence (whitelist mode)
2. `--exclude` removes from result (blacklist mode)
3. Multiple `--exclude` values are OR'd (exclude if any match)
4. Multiple `--only` values are OR'd (include if any match)
5. `-t/--type` applies to symbols, not files

### 2. Interaction with Existing Flags

- `--types-only` is sugar for `-t class,struct,enum,interface,type`
- `--include-private` is orthogonal to all filters (controls visibility, not selection)
- `--focus` and `--resolve-imports` work on filtered result

### 3. Output Indication

When filters are active, indicate in output:
```
src/ (filtered: --exclude="*_test*")
├── lib/
├── main.rs
└── api/
    └── ...
```

## Implementation Notes

- Filters apply during tree traversal, not post-processing
- Glob patterns use gitignore-style matching (same as `ignore` crate)

## Cross-Command Unification

### Proposed Shared Flags

These flags should work identically across view, analyze, and future batch commands:

```
-t, --type <KIND,...>   Symbol kind filter (rename analyze's --kind)
--exclude <GLOB>        Exclude matching paths (repeatable)
--only <GLOB>           Include only matching paths (repeatable)
```

(If we go with Option E, add `@name` syntax for config-defined aliases.)

### Command-Specific Behavior

| Command | How filters apply |
|---------|-------------------|
| view | Filters tree nodes before display |
| analyze | Filters files/symbols before analysis |
| edit | N/A (operates on specific target) |
| grep | Could add `--exclude` for file filtering |
| lint | Could add `--exclude` for file filtering |

### Migration

1. Add `--type` alias to analyze's `--kind` (deprecate `--kind`)
2. Add shared filters to view first
3. Propagate to analyze, grep, lint

## Not Included (Too Complex)

- Regex filters (glob is sufficient, regex is overkill)
- Content-based filters (grep exists for that)
- Complex boolean expressions (use multiple commands with jq)
