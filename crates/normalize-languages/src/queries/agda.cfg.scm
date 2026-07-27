; Agda CFG query
; Captures control flow nodes for CFG construction.
; See normalize-cfg for the full capture vocabulary.
; Verified against arborium Agda grammar node types.
;
; Agda is a dependently-typed proof assistant / functional language.
; Control flow is purely via pattern matching in function definitions
; (each `lhs = rhs` clause is a branch alternative). There is no
; if-then-else syntax node in the grammar (if-then-else is a library
; function, not syntax) and no loops, breaks, or throws. `lambda_clause`
; (anonymous-function bodies) has no `lhs`/`rhs` children — just an
; optional pattern application, an arrow, and an `expr` — so it carries
; no distinct CFG structure worth capturing here.

; ---------------------------------------------------------------------------
; Pattern matching in function definitions (branch-like)
; ---------------------------------------------------------------------------

; Each function clause with pattern matching is a branch
(function
  (lhs) @cfg.branch.condition
  (rhs) @cfg.branch.then
) @cfg.branch
