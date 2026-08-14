; Agda tags query
;
; MAJOR CORRECTNESS FIX (verified via `normalize syntax ast`/`normalize
; syntax query --show-source`, not node-types.json alone — node-types.json
; is misleading here, see below): a `function` node's `lhs` has two
; distinct real shapes:
;   - Type-signature form (`f : T`, and a data constructor's declaration
;     `Circle : Shape`): `lhs > function_name > atom > qid`.
;   - Defining-equation form (`f x y = ...`, including point-free `f =
;     ...`): `lhs`'s name is a BARE `atom` — no `function_name` wrapper at
;     all.
; The previous query only matched `(lhs (function_name) @name)`, which
; ONLY ever matches the signature form. Every function's actual defining
; equation was completely unmatched, and any function written without an
; explicit type signature (legal, and common during incremental/proof-in-
; progress development) was entirely invisible to tags.
;
; The obvious fix — `(lhs (atom) @name)` — does not compile: tree-sitter's
; query validator rejects it as an "Impossible pattern" even though
; node-types.json lists `atom` among `lhs`'s allowed children and the real
; parse tree visibly contains it (confirmed via `normalize syntax ast`).
; This is the grammar behaving exactly as
; docs/query-testing-methodology.md warns: node-types.json can list
; positions the compiled grammar's own query-validation tables disagree
; with. Anchoring on the wildcard first child instead —
; `(lhs . (_) @name)` — sidesteps the restriction and is verified to
; return the correct identifier text for BOTH shapes uniformly (the
; `function_name`/`atom` wrapper nodes in this position never contain
; anything but the bare name), so one pattern now covers both forms
; instead of two.
;
; Both the signature and the equation are kept as separate definition
; sites (not deduplicated the way haskell.tags.scm intentionally excludes
; type signatures) because, unlike Haskell, an Agda function very commonly
; exists as signature-only for a while during interactive/incremental
; proof development — dropping the signature-only case would make such
; functions invisible until their body is filled in.
;
; SCOPING: explicitly anchored to the containers that hold real top-level
; (or nested-module/private/abstract/instance/mutual-block) declarations —
; `source_file` (top-level module's own declarations attach directly to
; `source_file`, NOT to the `module` node itself: the `module` node for a
; file's own `module X where` header has no declaration children, only
; `module`/`module_name`/`where` — verified via `normalize syntax ast`),
; `module` (a nested/inner submodule DOES have declaration children), and
; `private`/`abstract`/`instance`/`mutual` (declaration-grouping blocks).
; `function` is ALSO the node type for `where`-bound local helper
; definitions (`outer n = f n where f x = x`), which parse as
; `function > where > function` — nested one level below the outer
; equation's own `function`, never a direct child of any of the containers
; above — so an unscoped pattern (which the previous query effectively
; was, for the one shape it did match) leaks locals in as top-level
; symbols. Confirmed via `normalize syntax ast` on a `where`-clause probe:
; without this scoping, a `where`-bound `helper`'s signature was captured
; as a top-level symbol.
(source_file
  (function
    (lhs
      .
      (_) @name))) @definition.function

(module
  (function
    (lhs
      .
      (_) @name))) @definition.function

(private
  (function
    (lhs
      .
      (_) @name))) @definition.function

(abstract
  (function
    (lhs
      .
      (_) @name))) @definition.function

(instance
  (function
    (lhs
      .
      (_) @name))) @definition.function

(mutual
  (function
    (lhs
      .
      (_) @name))) @definition.function

; Data constructor declarations: `Circle : Shape` inside `data Shape
; where`. Structurally identical (`function > lhs > function_name`) to a
; top-level type signature — the grammar has no distinct "constructor"
; node type — so this reuses the same pattern shape, scoped to `data`
; instead of a top-level container. Constructors are captured as
; definition.function rather than a fabricated definition.constructor kind
; since this codebase's tags.scm vocabulary (verified: grep across
; crates/normalize-languages/src/queries/*.tags.scm) has no constructor
; category.
(data
  (function
    (lhs
      .
      (_) @name))) @definition.function

; NOTE: no dedicated pattern for record field signatures (`field x : Int`
; inside `record ... where`) — this codebase's tags.scm convention across
; languages (verified: rust.tags.scm, go.tags.scm) does not tag individual
; struct/record fields as top-level definitions, only functions/types/
; classes/modules. A previous pattern here (`(signature (signature
; (function_name) @name)) @definition.function`) attempted to capture this
; case but was dead code on two counts: node-types.json shows record field
; signatures nest as `signature > field_name`, not `signature > signature
; > function_name`, and no such double-nested `signature` shape is ever
; produced by the grammar for any construct (confirmed 0 matches for a
; direct `(signature (signature ...))` probe against sample.agda).

; Data type declarations
(data
  (data_name) @name) @definition.type

; Record type declarations
(record
  (record_name) @name) @definition.class

; Module declarations
(module
  (module_name) @name) @definition.module
