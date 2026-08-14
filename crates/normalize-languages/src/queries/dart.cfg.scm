; Dart CFG query
; Captures control flow nodes for CFG construction.
; See normalize-cfg for the full capture vocabulary.
; Verified against arborium Dart grammar node types.
;
; `if_statement` has no `condition:` field — only `consequence:`/
; `alternative:` are fielded; the condition is an unfielded child anchored
; right after the literal "if" token. `for_statement`'s `condition:`/
; `value:` fields live one level down, inside the (named, non-hidden)
; `for_loop_parts` node — not directly on `for_statement` itself.

; ---------------------------------------------------------------------------
; if / else (branch)
; ---------------------------------------------------------------------------

(if_statement
  .
  "if"
  .
  (_) @cfg.branch.condition
  .
  consequence: (_) @cfg.branch.then
  alternative: (_) @cfg.branch.else
) @cfg.branch

(if_statement
  .
  "if"
  .
  (_) @cfg.branch.condition
  .
  consequence: (_) @cfg.branch.then
) @cfg.branch

; ---------------------------------------------------------------------------
; switch (match)
; ---------------------------------------------------------------------------

(switch_statement
  condition: (_) @cfg.match.scrutinee
  body: (switch_block
    (switch_statement_case) @cfg.match.arm
  )
) @cfg.match

; Dart 3 pattern-matching switch expression: `switch (n) { 0 => ..., _ => ... }`
; — distinct node type from switch_statement (an expression, not a
; statement), with switch_expression_case children directly under the
; repeated `body` field (no intermediate switch_block wrapper).
(switch_expression
  condition: (_) @cfg.match.scrutinee
  body: (switch_expression_case) @cfg.match.arm
) @cfg.match

; ---------------------------------------------------------------------------
; for (C-style loop)
; ---------------------------------------------------------------------------

(for_statement
  (for_loop_parts
    condition: (_) @cfg.loop.condition)
  body: (_) @cfg.loop.body
) @cfg.loop

; ---------------------------------------------------------------------------
; for-in (loop over collection)
; ---------------------------------------------------------------------------

(for_statement
  (for_loop_parts
    value: (_) @cfg.loop.condition)
  body: (_) @cfg.loop.body
) @cfg.loop

; ---------------------------------------------------------------------------
; while (loop with condition)
; ---------------------------------------------------------------------------

(while_statement
  condition: (_) @cfg.loop.condition
  body: (_) @cfg.loop.body
) @cfg.loop

; ---------------------------------------------------------------------------
; do-while (loop with condition at end)
; ---------------------------------------------------------------------------

(do_statement
  body: (_) @cfg.loop.body
  condition: (_) @cfg.loop.condition
) @cfg.loop

; ---------------------------------------------------------------------------
; try / catch / finally
; ---------------------------------------------------------------------------

(try_statement
  body: (_) @cfg.try.body
) @cfg.try

(catch_clause) @cfg.try.catch

(finally_clause) @cfg.try.finally

; ---------------------------------------------------------------------------
; Exits
; ---------------------------------------------------------------------------

(return_statement) @cfg.exit.return

(break_statement) @cfg.exit.break

(continue_statement) @cfg.exit.continue

(throw_expression) @cfg.exit.throw
