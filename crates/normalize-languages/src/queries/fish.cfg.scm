; Fish shell CFG query
; Captures control flow nodes for CFG construction.
; See normalize-cfg for the full capture vocabulary.
; Verified against arborium Fish grammar node types via `normalize syntax ast`.
;
; `if_statement`, `for_statement`, and `while_statement` have no `body:`
; field — the branch/loop body is one or more unfielded statement children
; following the anonymous keyword/condition. The anchor below binds to the
; single statement immediately following the condition (mirrors the same
; grammar shape in Bash).

; ---------------------------------------------------------------------------
; if / else if / else (branch)
; ---------------------------------------------------------------------------

(if_statement
  condition: (_) @cfg.branch.condition
  .
  (_) @cfg.branch.then
) @cfg.branch

(if_statement
  (else_clause) @cfg.branch.else
) @cfg.branch

(if_statement
  (else_if_clause) @cfg.branch.else
) @cfg.branch

; ---------------------------------------------------------------------------
; switch / case (match)
; ---------------------------------------------------------------------------

(switch_statement
  value: (_) @cfg.match.scrutinee
  (case_clause) @cfg.match.arm
) @cfg.match

; ---------------------------------------------------------------------------
; for (loop)
; ---------------------------------------------------------------------------

(for_statement
  value: (_) @cfg.loop.condition
  .
  (_) @cfg.loop.body
) @cfg.loop

; ---------------------------------------------------------------------------
; while (loop with condition)
; ---------------------------------------------------------------------------

(while_statement
  condition: (_) @cfg.loop.condition
  .
  (_) @cfg.loop.body
) @cfg.loop

; ---------------------------------------------------------------------------
; Exits
; ---------------------------------------------------------------------------

(return) @cfg.exit.return

(break) @cfg.exit.break

(continue) @cfg.exit.continue
