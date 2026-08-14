; Julia CFG query
; Captures control flow nodes for CFG construction.
; See normalize-cfg for the full capture vocabulary.
; Verified against arborium Julia grammar node types and against
; crates/normalize-languages/tests/fixtures/julia/sample.jl via
;   normalize syntax query <query> -p <file> --show-source
;
; if_statement has a "condition" field but no "consequence" field —
; the then-branch is an unnamed (block) child. "alternative" (multiple,
; optional) holds else_clause / elseif_clause. for_statement and
; try_statement have no named fields at all: their children (block,
; for_binding) / (block, catch_clause, else_clause, finally_clause) are
; unnamed and matched positionally/by type. call_expression likewise has
; no "function" field — the callee is the first unnamed child.

; ---------------------------------------------------------------------------
; if / elseif / else (branch)
; ---------------------------------------------------------------------------

; alternative: is a "multiple: true" field (if_statement can carry several
; elseif_clause siblings followed by an optional final else_clause, all
; flattened directly under if_statement rather than nested inside each
; other — confirmed via `normalize syntax ast`). The `.` anchor between
; (block) and `alternative:` requires the alternative to be the *first*
; sibling immediately following the then-block, so this fires exactly once
; per if_statement regardless of whether that first alternative is an
; elseif_clause or an else_clause — verified via `normalize syntax query`
; against an elseif+elseif+else chain (single match, first alternative only)
; and an elseif-only chain with no final else (previously unmatched: see
; below).
;
; PRIOR BUG (fixed here): the old query only matched `alternative:
; (else_clause)`, so `if a ... elseif b ... end` (no trailing else) never
; produced a @cfg.branch for the outer if's own condition/then at all —
; confirmed via `normalize syntax query` returning 0 matches for that shape
; under both of the old patterns. The elseif_clause itself still got its own
; @cfg.branch via the separate (elseif_clause ...) pattern below, but the
; outer if's initial branch was invisible to CFG construction.
(if_statement
  condition: (_) @cfg.branch.condition
  (block) @cfg.branch.then
  .
  alternative: (_) @cfg.branch.else
) @cfg.branch

(if_statement
  condition: (_) @cfg.branch.condition
  (block) @cfg.branch.then
  .
) @cfg.branch

; elseif clause (branch within if chain)
(elseif_clause
  condition: (_) @cfg.branch.condition
  (block) @cfg.branch.then
) @cfg.branch

; ---------------------------------------------------------------------------
; for (loop)
; ---------------------------------------------------------------------------

(for_statement
  (for_binding) @cfg.loop.condition
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
; try / catch / finally
; ---------------------------------------------------------------------------

(try_statement
  (block) @cfg.try.body
) @cfg.try

(catch_clause) @cfg.try.catch

(finally_clause) @cfg.try.finally

; ---------------------------------------------------------------------------
; Exits
; ---------------------------------------------------------------------------

(return_statement) @cfg.exit.return

(break_statement) @cfg.exit.break

(continue_statement) @cfg.exit.continue

; throw() is the throw equivalent in Julia
(call_expression
  (identifier) @_fn
  (#match? @_fn "^(throw|error|rethrow)$")
) @cfg.exit.throw
