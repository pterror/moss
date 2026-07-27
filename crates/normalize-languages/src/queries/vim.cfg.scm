; Vim script CFG query
; Captures control flow nodes for CFG construction.
; See normalize-cfg for the full capture vocabulary.
; Verified against arborium Vim grammar node types using real fixtures.
; Note: the branch/loop body is a named `(body)` node, not a `body:` field.
; `for_loop`'s iterable is the `iter:` field, not `variable:` (that field
; holds the loop variable itself).

; ---------------------------------------------------------------------------
; if / elseif / else (branch)
; ---------------------------------------------------------------------------

(if_statement
  condition: (_) @cfg.branch.condition
  (body) @cfg.branch.then
  (elseif_statement) @cfg.branch.else
) @cfg.branch

(if_statement
  condition: (_) @cfg.branch.condition
  (body) @cfg.branch.then
  (else_statement
    (body) @cfg.branch.else)
) @cfg.branch

(if_statement
  condition: (_) @cfg.branch.condition
  (body) @cfg.branch.then
  .
) @cfg.branch

; ---------------------------------------------------------------------------
; for (loop)
; ---------------------------------------------------------------------------

(for_loop
  iter: (_) @cfg.loop.condition
  (body) @cfg.loop.body
) @cfg.loop

; ---------------------------------------------------------------------------
; while (loop with condition)
; ---------------------------------------------------------------------------

(while_loop
  condition: (_) @cfg.loop.condition
  (body) @cfg.loop.body
) @cfg.loop

; ---------------------------------------------------------------------------
; try / catch / finally
; ---------------------------------------------------------------------------

(try_statement
  (body) @cfg.try.body
) @cfg.try

(catch_statement) @cfg.try.catch

(finally_statement) @cfg.try.finally

; ---------------------------------------------------------------------------
; Exits
; ---------------------------------------------------------------------------

(return_statement) @cfg.exit.return

(break_statement) @cfg.exit.break

(continue_statement) @cfg.exit.continue

(throw_statement) @cfg.exit.throw
