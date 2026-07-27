; Visual Basic .NET CFG query
; Captures control flow nodes for CFG construction.
; See normalize-cfg for the full capture vocabulary.
; Verified against arborium VB.NET grammar node types using real fixtures.
; Note: branch/loop bodies are plain `(statement)` children, not a `body:`
; field. `if_statement` has no `else_clause`-wrapping-`body:` shape — the
; then-body is a direct `(statement)` sibling and `elseif_clause`/
; `else_clause` are separate sibling nodes. `for_statement` has no
; `for_to_clause` wrapper (start/end are direct fields). The Do/Loop node is
; `do_statement`, not `do_loop_statement`. Try/catch/finally use
; `catch_block`/`finally_block`, not `*_clause`.

; ---------------------------------------------------------------------------
; If / ElseIf / Else (branch)
; ---------------------------------------------------------------------------

(if_statement
  condition: (_) @cfg.branch.condition
  (statement) @cfg.branch.then
  .
) @cfg.branch

(if_statement
  condition: (_) @cfg.branch.condition
  (statement) @cfg.branch.then
  (elseif_clause) @cfg.branch.else
) @cfg.branch

(if_statement
  condition: (_) @cfg.branch.condition
  (statement) @cfg.branch.then
  (else_clause) @cfg.branch.else
) @cfg.branch

; ---------------------------------------------------------------------------
; Select Case (match)
; ---------------------------------------------------------------------------

(select_case_statement
  selector: (_) @cfg.match.scrutinee
  (case_block) @cfg.match.arm
) @cfg.match

(select_case_statement
  selector: (_) @cfg.match.scrutinee
  (case_else_block) @cfg.match.arm
) @cfg.match

; ---------------------------------------------------------------------------
; For / For Each (loop)
; ---------------------------------------------------------------------------

(for_statement
  start: (_) @cfg.loop.condition
  end: (_)
  (statement) @cfg.loop.body
) @cfg.loop

(for_each_statement
  collection: (_) @cfg.loop.condition
  (statement) @cfg.loop.body
) @cfg.loop

; ---------------------------------------------------------------------------
; While / Do (loop)
; ---------------------------------------------------------------------------

(while_statement
  condition: (_) @cfg.loop.condition
  (statement) @cfg.loop.body
) @cfg.loop

(do_statement) @cfg.loop

; ---------------------------------------------------------------------------
; Try / Catch / Finally
; ---------------------------------------------------------------------------

(try_statement
  (statement) @cfg.try.body
) @cfg.try

(catch_block) @cfg.try.catch

(finally_block) @cfg.try.finally

; ---------------------------------------------------------------------------
; Exits
; ---------------------------------------------------------------------------

(return_statement) @cfg.exit.return

(exit_statement) @cfg.exit.break

(continue_statement) @cfg.exit.continue

(throw_statement) @cfg.exit.throw
