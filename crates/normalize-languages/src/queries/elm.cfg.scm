; Elm CFG query
; Captures control flow nodes for CFG construction.
; See normalize-cfg for the full capture vocabulary.
; Verified against arborium Elm grammar node types.
;
; Elm is a purely functional language. Control flow is via if-else
; expressions and case-of expressions (pattern matching).
; There are no loops, break, continue, or throw.
;
; `if_else_expr` has no `expr`-typed children — the condition/then/else
; are all `exprList:`-fielded, and (for an `else if` chain) they're flat,
; repeated siblings rather than nested. Anchors tie each condition to its
; immediately-following then-branch, and the trailing anchor after `else`
; restricts the full then+else pattern to a chain with no further
; `else if` links. `case_of_expr` uses `expr:`/`branch:` fields, not
; bare `(expr)` children.

; ---------------------------------------------------------------------------
; if / else (branch expression)
; ---------------------------------------------------------------------------

; Full if/then/else with no further `else if` chaining.
(if_else_expr
  .
  exprList: (_) @cfg.branch.condition
  .
  exprList: (_) @cfg.branch.then
  .
  exprList: (_) @cfg.branch.else
  .
) @cfg.branch

; `else if` chain: each condition/then pair is its own branch (mirrors
; how `elsif` is handled for other flat-chain grammars in this codebase).
(if_else_expr
  .
  exprList: (_) @cfg.branch.condition
  .
  exprList: (_) @cfg.branch.then
) @cfg.branch

; ---------------------------------------------------------------------------
; case / of (match)
; ---------------------------------------------------------------------------

(case_of_expr
  expr: (_) @cfg.match.scrutinee
  branch: (case_of_branch) @cfg.match.arm
) @cfg.match
