; D CFG query
; Captures control flow nodes for CFG construction.
; See normalize-cfg for the full capture vocabulary.
; Verified against arborium D grammar node types.
;
; D's statement grammar is almost entirely unfielded: `if_statement`,
; `while_statement`, `for_statement`, `switch_statement`, `do_statement`,
; `try_statement` have no `condition:`/`body:`/`thenStatement:` fields —
; positions are matched structurally instead. `then_statement`/
; `else_statement` *are* real (non-hidden) node types despite wrapping a
; hidden `_scope_statement`, so they can be matched directly by type.

; ---------------------------------------------------------------------------
; if / else (branch)
; ---------------------------------------------------------------------------

(if_statement
  [(expression) (if_condition)] @cfg.branch.condition
  (then_statement) @cfg.branch.then
  (else_statement) @cfg.branch.else
) @cfg.branch

(if_statement
  [(expression) (if_condition)] @cfg.branch.condition
  (then_statement) @cfg.branch.then
) @cfg.branch

; ---------------------------------------------------------------------------
; switch (match)
; ---------------------------------------------------------------------------

(switch_statement
  (expression) @cfg.match.scrutinee
  (block_statement
    (statement_list
      (case_statement) @cfg.match.arm))
) @cfg.match

; ---------------------------------------------------------------------------
; for / foreach (loop)
; ---------------------------------------------------------------------------

(for_statement
  (test) @cfg.loop.condition
  (block_statement) @cfg.loop.body
) @cfg.loop

(for_statement
  (block_statement) @cfg.loop.body
) @cfg.loop

(foreach_statement
  (aggregate_foreach
    (foreach_type_list) @cfg.loop.condition)
  (block_statement) @cfg.loop.body
) @cfg.loop

; ---------------------------------------------------------------------------
; while (loop with condition)
; ---------------------------------------------------------------------------

(while_statement
  (expression) @cfg.loop.condition
  .
  (block_statement) @cfg.loop.body
) @cfg.loop

; ---------------------------------------------------------------------------
; do-while (loop with condition at end)
; ---------------------------------------------------------------------------

(do_statement
  .
  (block_statement) @cfg.loop.body
  (expression) @cfg.loop.condition
) @cfg.loop

; ---------------------------------------------------------------------------
; try / catch / finally
; ---------------------------------------------------------------------------

(try_statement
  .
  (block_statement) @cfg.try.body
) @cfg.try

(catch) @cfg.try.catch

(finally_statement) @cfg.try.finally

; ---------------------------------------------------------------------------
; Exits
; ---------------------------------------------------------------------------

(return_statement) @cfg.exit.return

(break_statement) @cfg.exit.break

(continue_statement) @cfg.exit.continue

(throw_statement) @cfg.exit.throw

(goto_statement) @cfg.exit.throw
