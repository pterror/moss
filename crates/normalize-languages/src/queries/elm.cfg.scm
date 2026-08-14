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
; repeated siblings: `if C1 then T1 else if C2 then T2 else if C3 then T3
; else E` is a SINGLE `if_else_expr` node with 2N+1 `exprList` children
; (N condition/then pairs + a final else), not N nested `if_else_expr`
; nodes the way e.g. Python nests `elif` in the `alternative` field.
;
; Correctness bug fixed here (verified via `normalize syntax query`): the
; previous two-pattern version — a fully-anchored 3-`exprList` pattern for
; the non-chained case, plus a *leading-anchored-only* 2-`exprList` pattern
; meant to catch each pair of an `else if` chain — could only ever bind to
; the FIRST `exprList` pair of a chain (the leading `.` anchor pins
; `@cfg.branch.condition` to the node's first child, always), so a chain of
; 2+ `else if` links silently dropped every middle condition/then pair
; entirely, capturing only the first `if`'s pair and the trailing `else`
; from the two patterns combined — a real, silent under-count.
;
; Since none of the flat `exprList` children carry a distinguishing field
; name (there's no way to say "match only the 3rd/5th slot" declaratively
; in tree-sitter query syntax — no index predicates), the fix is one fully
; both-ends-anchored pattern PER exact `exprList` count (2N+1 for N
; branches), each requiring the WHOLE node to have exactly that many
; children so the patterns can't overlap or produce a sliding-window
; mismatch (which happens if you drop the anchors: `cond`/`then` fields
; then also match `then_i` paired with `cond_{i+1}` as a bogus pair —
; verified and rejected). Patterns are provided for chains of 1–4 `else
; if`-linked branches (up to 5-way branching); a chain with a 5th `else
; if` link goes unmatched entirely — a real but very rare style Elm's own
; formatter/community conventions discourage (favoring `case ... of`
; instead of long `if`/`else if` chains past a couple of branches). This is
; a query-expressiveness limit (no arbitrary-arity repetition in
; tree-sitter query syntax for same-named flat fields), not a grammar
; limitation — the CST represents arbitrarily long chains fine.

; ---------------------------------------------------------------------------
; if / else (branch expression)
; ---------------------------------------------------------------------------

; 1 branch: if C then T else E  (3 exprList children)
(if_else_expr
  .
  exprList: (_) @cfg.branch.condition
  .
  exprList: (_) @cfg.branch.then
  .
  exprList: (_) @cfg.branch.else
  .
) @cfg.branch

; 2 branches: if C1 then T1 else if C2 then T2 else E  (5 exprList children)
(if_else_expr
  .
  exprList: (_) @cfg.branch.condition
  .
  exprList: (_) @cfg.branch.then
  .
  exprList: (_) @cfg.branch.condition
  .
  exprList: (_) @cfg.branch.then
  .
  exprList: (_) @cfg.branch.else
  .
) @cfg.branch

; 3 branches (7 exprList children)
(if_else_expr
  .
  exprList: (_) @cfg.branch.condition
  .
  exprList: (_) @cfg.branch.then
  .
  exprList: (_) @cfg.branch.condition
  .
  exprList: (_) @cfg.branch.then
  .
  exprList: (_) @cfg.branch.condition
  .
  exprList: (_) @cfg.branch.then
  .
  exprList: (_) @cfg.branch.else
  .
) @cfg.branch

; 4 branches (9 exprList children)
(if_else_expr
  .
  exprList: (_) @cfg.branch.condition
  .
  exprList: (_) @cfg.branch.then
  .
  exprList: (_) @cfg.branch.condition
  .
  exprList: (_) @cfg.branch.then
  .
  exprList: (_) @cfg.branch.condition
  .
  exprList: (_) @cfg.branch.then
  .
  exprList: (_) @cfg.branch.condition
  .
  exprList: (_) @cfg.branch.then
  .
  exprList: (_) @cfg.branch.else
  .
) @cfg.branch

; ---------------------------------------------------------------------------
; case / of (match)
; ---------------------------------------------------------------------------

(case_of_expr
  expr: (_) @cfg.match.scrutinee
  branch: (case_of_branch) @cfg.match.arm
) @cfg.match
