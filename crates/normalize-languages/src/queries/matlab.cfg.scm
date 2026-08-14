; MATLAB CFG query
; Captures control flow nodes for CFG construction.
; See normalize-cfg for the full capture vocabulary.
; Verified against arborium MATLAB grammar node types and against
; crates/normalize-languages/tests/fixtures/matlab/sample.m via the
; GrammarLoader directly (NOT `normalize syntax ast/query -p <file>` —
; MATLAB's ".m" extension collides with Objective-C's, and the CLI's
; extension-based language detection resolves fixtures/matlab/sample.m
; to the objc grammar; see TODO.md for that pre-existing bug).
;
; if_statement has a "condition" field but no "body" field — the
; then-branch is an unnamed (block) child, with optional unnamed
; (elseif_clause)/(else_clause) siblings. for_statement has no fields
; at all: its unnamed children are (iterator) and (block).
;
; KNOWN LIMITATION (documented, not silently accepted): MATLAB's
; `elseif`/`else` clauses are FLAT siblings of `if_statement`, not nested
; recursively (unlike e.g. Python, where each `elif` is itself a nested
; `if_statement` in the `alternative` field). Because of this flat shape,
; an `if` with 2+ `elseif_clause` children (with or without a trailing
; `else_clause`) makes multiple of the three patterns below match the SAME
; `if_statement` node — once per elseif/else arm. `normalize-cfg`'s builder
; (`crates/normalize-cfg/src/builder.rs`) deduplicates structural matches by
; the primary node's start byte (`structural_nodes.dedup_by_key(|n|
; n.byte_range.start)`), since it assumes at most one `@cfg.branch` match per
; branch node. For a flat elseif chain this silently KEEPS ONLY THE FIRST
; matching arm and DROPS every later `elseif`/`else` arm's statements from
; the CFG entirely (verified: `if n>0 ... elseif n==0 ... elseif n<-10 ...
; else ... end` produces 3 overlapping `@cfg.branch` matches for the one
; `if_statement`, each with a different `@cfg.branch.else`; only the first,
; `elseif n==0 ...`, survives dedup — the `elseif n<-10` and `else` arms are
; never visited by the builder). A single `elseif` with no further
; elseif/else, or a single `else` with no elseif, is unaffected (only one
; pattern matches). This is a `normalize-cfg` builder limitation, not a
; grammar or query-completeness gap — the query correctly captures every
; arm's condition/then/else; the builder's singular `branch_else: Option<_>`
; field has no representation for a 3+-way branch chain. The same class of
; grammar (flat elseif children) affects `bash.cfg.scm`. Fixing this
; requires builder-level support for multi-arm branches, not a per-language
; query change — see TODO.md.

; ---------------------------------------------------------------------------
; if / elseif / else (branch)
; ---------------------------------------------------------------------------

(if_statement
  condition: (_) @cfg.branch.condition
  (block) @cfg.branch.then
  (elseif_clause) @cfg.branch.else
) @cfg.branch

(if_statement
  condition: (_) @cfg.branch.condition
  (block) @cfg.branch.then
  (else_clause) @cfg.branch.else
) @cfg.branch

(if_statement
  condition: (_) @cfg.branch.condition
  (block) @cfg.branch.then
  .
) @cfg.branch

; ---------------------------------------------------------------------------
; switch / case (match)
; ---------------------------------------------------------------------------

(switch_statement
  condition: (_) @cfg.match.scrutinee
  (case_clause) @cfg.match.arm
) @cfg.match

; ---------------------------------------------------------------------------
; for (loop)
; ---------------------------------------------------------------------------

(for_statement
  (iterator) @cfg.loop.condition
  (block) @cfg.loop.body
) @cfg.loop

; ---------------------------------------------------------------------------
; while (loop with condition)
; ---------------------------------------------------------------------------

(while_statement
  condition: (_) @cfg.loop.condition
  (block) @cfg.loop.body
) @cfg.loop

; ---------------------------------------------------------------------------
; try / catch (exception handling)
; ---------------------------------------------------------------------------

(try_statement
  (block) @cfg.try.body
) @cfg.try

(catch_clause) @cfg.try.catch

; ---------------------------------------------------------------------------
; Exits
; ---------------------------------------------------------------------------

(return_statement) @cfg.exit.return

(break_statement) @cfg.exit.break

(continue_statement) @cfg.exit.continue

; error() is the throw equivalent in MATLAB
(function_call
  name: (identifier) @_fn
  (#match? @_fn "^(error|rethrow)$")
) @cfg.exit.throw
