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

(if_statement
  condition: (_) @cfg.branch.condition
  (block) @cfg.branch.then
  alternative: (else_clause) @cfg.branch.else
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
