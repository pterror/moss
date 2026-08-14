; Go CFG query
; Captures control flow nodes for CFG construction.
; See normalize-cfg for the full capture vocabulary.
; Note: verified against the arborium Go grammar tree-sitter node types.

; ---------------------------------------------------------------------------
; if (branch)
; ---------------------------------------------------------------------------

(if_statement
  consequence: (_) @cfg.branch.then
  alternative: (_) @cfg.branch.else
) @cfg.branch

(if_statement
  consequence: (_) @cfg.branch.then
  .
  ; no alternative
) @cfg.branch

; ---------------------------------------------------------------------------
; switch / expression_switch (match)
; ---------------------------------------------------------------------------

(expression_switch_statement
  value: (_) @cfg.match.scrutinee
  (expression_case) @cfg.match.arm
) @cfg.match

(expression_switch_statement
  (expression_case) @cfg.match.arm
) @cfg.match

; ---------------------------------------------------------------------------
; for (loop — covers for-range, for-condition, three-clause, and
; unconditional forms)
; ---------------------------------------------------------------------------
;
; Go's `for_statement` has no `condition:` field (verified via `normalize
; syntax query`: `(for_statement condition: (_) @c)` is an "Impossible
; pattern" against this grammar) — its shape instead varies by which node,
; if any, precedes the `block`: a bare `for_clause` (three-clause form,
; e.g. `for i := 0; i < n; i++ { }`), a `range_clause` (e.g. `for _, item
; := range items { }`), a plain boolean expression (condition-only form,
; e.g. `for i < n { }`), or nothing at all (unconditional `for { }`).
;
; Without a @cfg.loop.condition capture, the CFG builder
; (normalize-cfg::builder::build_loop) treats every Go `for` as
; unconditional: `LoopHead` falls straight through to `LoopBody` with no
; `true`/`false` edges, and `LoopExit` becomes reachable only via `break` —
; orphaned for any range/condition/three-clause loop with no `break`.
; Verified live: `go_cfg__go_loop.snap`'s `for _, item := range items`
; loop showed exactly this shape before the fix (no labeled edges into
; `LoopExit`), matching the same class of bug fixed in jinja2.cfg.scm
; (commit a8f316f1) — the diagnostic there was a missing/duplicated
; @cfg.loop.condition/@cfg.loop.body capture leaving the builder unable to
; tell a conditional loop from an unconditional one.
;
; The two patterns below are structurally mutually exclusive (confirmed via
; `normalize syntax query` against all four forms): the first requires a
; named node immediately before `block` (true for for_clause/range_clause/
; condition-only), the second requires `block` to be the first named child
; (true only for bare `for { }`).

(for_statement
  (_) @cfg.loop.condition
  .
  (block) @cfg.loop.body
) @cfg.loop

(for_statement
  .
  (block) @cfg.loop.body
) @cfg.loop

; ---------------------------------------------------------------------------
; Exits
; ---------------------------------------------------------------------------

(return_statement) @cfg.exit.return

(break_statement) @cfg.exit.break

(continue_statement) @cfg.exit.continue

; ---------------------------------------------------------------------------
; Def/use sites
; ---------------------------------------------------------------------------

; y := expr — short variable declaration
(short_var_declaration
  left: (expression_list
    (identifier) @cfg.def.name
  )
) @cfg.def

; y = expr — assignment
(assignment_statement
  left: (expression_list
    (identifier) @cfg.def.name
  )
) @cfg.def

; ---------------------------------------------------------------------------
; Effects
; ---------------------------------------------------------------------------

; defer statement — deferred call registered, runs on function exit
(defer_statement) @cfg.effect.defer

; go statement — goroutine spawn (modelled as channel send analogue)
(go_statement) @cfg.effect.send

; send statement: ch <- val
(send_statement) @cfg.effect.send

; channel receive in expression context: <-ch
(unary_expression
  operator: "<-"
) @cfg.effect.receive
