# Query Testing Methodology

A mechanical checklist for taking a language's `.scm` query test coverage from
"doesn't panic" to genuinely thorough. Written after prototyping it on Rust
(`crates/normalize-languages/tests/fixtures/rust/{sample,variants}.rs` +
the `rust_*` tests in `crates/normalize-languages/tests/query_fixtures.rs`)
as a reusable pattern for the remaining ~95 supported languages.

**Context**: the 59-query remediation (`TODO.md`, "Query-file compile
guardrail") found queries that failed to *compile*. That's necessary but not
sufficient — a query can compile clean and still be wrong, in the same way
the original Lua bug was wrong: `function_declaration.name` could be
`identifier`, `dot_index_expression`, or `method_index_expression`, and the
query only matched one of the three. Compiling doesn't catch that; only
cross-referencing the query's field constraints against every node-type
variant the grammar actually allows does. This document is that process,
made repeatable.

## The four dimensions

1. **Query correctness** — the query compiles (covered by
   `all_registered_queries_compile`) AND its field constraints match real
   grammar nodes, verified against actual parse output, not memory or
   `node-types.json` alone.
2. **Query completeness** — for every field a query constrains
   (`function: (X)`, `name: (Y)`, `argument: (Z)`, …), does the query cover
   *every* node-type variant `node-types.json` allows for that field? This is
   the actual root cause of the original Lua bug, and of every real bug this
   prototype found for Rust (see below).
3. **Extraction depth** — assertions on capture *kind*, not just capture
   text (two different node kinds can produce identical text); full-count
   assertions with no unexplained duplicates; explicit negative cases
   (constructs that must NOT match, checked, not assumed).
4. **Real-world fixture coverage** — fixtures exercise the idioms the
   language's ecosystem actually leans on heavily (closures, generics,
   nested containers, decorators/attributes, multiple inheritance/trait
   patterns), not just the minimal toy construct needed to make one
   assertion pass.

## Mechanical checklist, per language

### 0. Inventory

- `ls crates/normalize-languages/src/queries/<lang>.*.scm` — which query
  purposes exist (tags, calls, imports, complexity, types, decorations, cfg,
  refactor, …).
- `ls crates/normalize-languages/tests/fixtures/<lang>/` — what fixture(s)
  already exist.
- `grep -n "^fn <lang>_" crates/normalize-languages/tests/query_fixtures.rs`
  — what's already tested, and how shallow it is (a single `.contains()`
  check is the shallow baseline this methodology replaces).

### 1. Find node-types.json

Grammars are vendored via arborium. Locate the crate source (not just the
compiled `.so`) under the Cargo registry:

```
find ~/.cargo/registry/src/index.crates.io-*/arborium-<lang>-*/ -iname node-types.json
```

Pick the version matching what's actually vendored in `Cargo.toml` /
`Cargo.lock` if multiple versions are cached. Parse it with `node -e '...'`
(this environment has no `python3`/`jq` by default — `node` is reliably
present via the Nix devshell) — dump the `fields` object for every node type
the query file constrains.

### 2. For each field constraint in the query, list every allowed variant

For a pattern like:

```scheme
(call_expression
  function: (identifier) @call)
```

look up `call_expression` in `node-types.json`, find the `function` field,
and read its full `types` array. In Rust this array had ~30 entries;
`identifier`, `scoped_identifier`, and `field_expression` (the three the
original query handled) were only 3 of them. `generic_function` — the node
wrapping any turbofish call (`func::<T>()`, `obj.method::<T>()`) — was
completely absent, silently dropping every turbofish call in the codebase
from extraction. Do this for every field-constrained pattern in the file,
not just the ones that look suspicious.

### 3. Verify each candidate variant against real parse output — never guess

Write a small probe file in the target language exercising the suspected gap,
then use the CLI itself (per CLAUDE.md's "verify before asserting" rule):

```
normalize syntax ast <probe-file>                      # see the full CST
normalize syntax query -p <probe-file> "(node_type field: (variant) @x)" --show-source
```

`normalize syntax query` compiles a *single* top-level pattern (not a whole
multi-pattern `.scm` file — feed it one candidate pattern at a time). Confirm
the variant actually appears in the CST with the field name you expect
before adding a query clause for it. Do not add a clause because
`node-types.json` says it's theoretically legal; confirm the grammar
actually produces that shape for realistic source, since node-types.json
occasionally lists positions that never actually arise in practice (or, per
some grammars found in the CFG remediation, fields that are declared but
never populated).

Cross-check real-world usage density with a grep over this very codebase
before deciding a gap is worth fixing vs. documenting as a rare edge case:

```
grep -rEn '<pattern for the idiom>' crates --include=*.rs | wc -l
```

A gap that fires hundreds of times in this repo's own source (as Rust's
turbofish-call and generic-impl gaps did) is a real, user-facing bug, not a
theoretical completeness nit.

### 4. Fix the query, keep the compile guardrail green

After editing, re-run:

```
NORMALIZE_REQUIRE_GRAMMARS=1 cargo test -p normalize-languages --test query_fixtures all_registered_queries_compile
```

This is fast (~2-3s) and catches syntax regressions immediately, before
writing a single test assertion.

### 5. Build (or extend) two fixtures, not one

- **`sample.rs`-equivalent** (idiomatic, real-world-shaped): extend the
  existing "toy" sample with the idioms dimension 4 asks for — closures,
  generics, nested modules/classes, decorators/attributes, multiple
  trait/interface implementations, iterator/pipeline chains. This is what
  makes correctness bugs visible in code that actually resembles what the
  language's users write.
- **`variants.rs`-equivalent** (completeness matrix): one small, clearly
  commented construct per node-type variant found in step 2, plus a
  dedicated NEGATIVE section with constructs that must *not* match (closures
  vs function definitions, bare field access vs a call, `let`-bound calls vs
  assignment-RHS calls). Comment each construct with which field/variant it
  exercises, so the fixture doubles as documentation of the completeness
  matrix — a future reader shouldn't have to re-derive why a given line
  exists.

Verify fixtures parse clean before writing tests against them:

```
normalize syntax query -p <fixture> "(ERROR) @err" --show-source   # must be 0 matches
```

### 6. Write tests across all four dimensions

Structure, per query purpose (tags/calls/imports/…):

- One test on the real-world sample asserting the idioms it demonstrates are
  found (dimension 4), reusing the crate's existing `collect_captures`
  helper.
- One `_completeness_*` test on the variants fixture asserting **every**
  variant from step 2 produces a capture with the correct **kind** (dimension
  2 + 3) — use `collect_captures_full` (returns `(capture_name, node_kind,
  text, line)`, not just text) so kind mismatches can't hide behind
  string-only assertions. Add `collect_captures_full` to
  `query_fixtures.rs`'s helper section if the language doesn't already have
  it available (it's crate-generic, not Rust-specific).
- At least one `_negative_*` test asserting the documented near-miss
  constructs produce zero matching captures, with exact counts (not just
  "is empty") where duplicates would otherwise hide a bug.
- For any query with a write/read distinction (calls' `@call` vs
  `@call.write`) or definition/reference distinction (tags' `@definition.*`
  vs `@reference.*`), a dedicated test asserting the distinction holds on
  both sides — not just that the busier side has entries.

### 7. Run the full gate before committing

```
cargo clippy --all-targets --all-features -- -D warnings && cargo test -q
```

Filter to the affected crate during iteration
(`cargo test -p normalize-languages -q`) but always run the full gate once
before committing — cross-crate consumers of a query file (budget metrics,
ratchet metrics, native rules, edit/refactor) can be affected by a
completeness fix that changes match counts.

### 8. Record what you found

- Real, user-facing extraction bugs found and fixed along the way go in
  `CHANGELOG.md` under `[Unreleased] Fixed`.
- Roadmap-level notes (which languages are done, which remain, links to the
  batch) go in `TODO.md`'s open-threads section, following the existing
  batch-tracking convention from the CFG remediation effort.
- Grammar-level absences (a construct the language's grammar genuinely
  cannot represent — no break/continue node type, no scoped-type field, etc.)
  get documented as a comment in the `.scm` file itself, not silently
  dropped and not fabricated. This mirrors the CFG remediation's own rule:
  "Be honest about capabilities" (CLAUDE.md).

## What this is not

This methodology does not itself schedule or batch the remaining ~95
languages — that's tracked as roadmap work in `TODO.md`. This document is
the reusable *how*, kept here (per CLAUDE.md's docs/ vs TODO.md split)
because it is stable reference material, not a per-session plan.

## Worked example: bugs found applying this to Rust

Applying steps 1–4 to `rust.{tags,calls,imports}.scm` found four real,
previously-silent extraction gaps (all now fixed, verified via
`normalize syntax query`/`normalize syntax ast`, and covered by the
completeness tests from step 6):

- **Turbofish calls entirely unmatched.** `call_expression.function` allows
  `generic_function` (wraps any `func::<T>()`/`obj.method::<T>()` call);
  neither `rust.calls.scm` nor `rust.tags.scm` handled it. 380+ call sites
  and 326+ method-call sites in this repo's own source use this idiom.
- **Generic and path-qualified impl blocks lost their container.**
  `impl_item.type`/`impl_item.trait` allow `generic_type` (`impl<T>
  Foo<T>`) and `scoped_type_identifier` (`impl std::fmt::Display for Foo`)
  in addition to plain `type_identifier`; `rust.tags.scm` only handled the
  plain form, so methods inside a generic or path-qualified impl weren't
  nested under their container. 47 generic impls and 131+ path-qualified
  impls in this repo's own source use these forms.
- **`self`-imports dropped.** `use path::{self, Other};` (bringing the
  module itself into scope alongside named members) is a distinct
  `use_list` child (`self`) that `rust.imports.scm` never matched — including
  in the pre-existing `sample.rs` fixture's own `use std::fmt::{self,
  Display};` line, which the original shallow test never exercised because
  it only checked for `HashMap`.
- **Bare wildcard imports (`use path::*;`) unmatched.** The query only
  handled the rarer braced form (`use path::{*};`); the far more common bare
  form parses as a structurally different tree (`use_wildcard` directly as
  the `use_declaration` argument, not nested inside `scoped_use_list` →
  `use_list`).
- **Scoped calls (`Type::method()`, `module::func()`) missing from tags.**
  `rust.tags.scm`'s `@reference.call` only had plain-identifier and
  method-call forms; the scoped form existed in `rust.calls.scm` but was
  never ported to `rust.tags.scm`.
