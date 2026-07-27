; Swift CFG query
; Captures control flow nodes for CFG construction.
; See normalize-cfg for the full capture vocabulary.
; Verified against arborium Swift grammar node types using real fixtures:
; `if_statement` has no `else_clause` wrapper — `condition` and the branch
; bodies are direct/field children, and the else arm is introduced by a
; named `else` node followed either by another `(statements)` block (final
; else) or a nested `if_statement` (else-if chain). Loop/branch bodies are
; plain `(statements)` nodes, not a `body:` field.

; ---------------------------------------------------------------------------
; if / guard / else (branch)
; ---------------------------------------------------------------------------

; if ... else if ... (chain continues via nested if_statement)
(if_statement
  condition: (_) @cfg.branch.condition
  (statements) @cfg.branch.then
  (else)
  (if_statement) @cfg.branch.else
) @cfg.branch

; if ... else { ... } (terminal else)
(if_statement
  condition: (_) @cfg.branch.condition
  (statements) @cfg.branch.then
  (else)
  (statements) @cfg.branch.else
) @cfg.branch

; if ... (no else)
(if_statement
  condition: (_) @cfg.branch.condition
  (statements) @cfg.branch.then
  .
) @cfg.branch

; guard (early exit — condition must be true to continue)
(guard_statement
  condition: (_) @cfg.branch.condition
  (statements) @cfg.branch.then
) @cfg.branch

; ---------------------------------------------------------------------------
; switch (match)
; ---------------------------------------------------------------------------

(switch_statement
  expr: (_) @cfg.match.scrutinee
  (switch_entry) @cfg.match.arm
) @cfg.match

; ---------------------------------------------------------------------------
; for-in (loop)
; ---------------------------------------------------------------------------

(for_statement
  collection: (_) @cfg.loop.condition
  (statements) @cfg.loop.body
) @cfg.loop

; ---------------------------------------------------------------------------
; while (loop with condition)
; ---------------------------------------------------------------------------

(while_statement
  condition: (_) @cfg.loop.condition
  (statements) @cfg.loop.body
) @cfg.loop

; ---------------------------------------------------------------------------
; repeat-while (do-while equivalent in Swift)
; ---------------------------------------------------------------------------

(repeat_while_statement
  (statements) @cfg.loop.body
  condition: (_) @cfg.loop.condition
) @cfg.loop

; ---------------------------------------------------------------------------
; do / catch (exception handling)
; ---------------------------------------------------------------------------

(do_statement
  (statements) @cfg.try.body
) @cfg.try

(catch_block) @cfg.try.catch

; ---------------------------------------------------------------------------
; Exits
;
; return/break/continue are anonymous tokens (not their own statement node
; types); throw is the named node `throw_keyword`. All are wrapped in a
; `control_transfer_statement`.
; ---------------------------------------------------------------------------

(control_transfer_statement "return" @cfg.exit.return)

(control_transfer_statement "break" @cfg.exit.break)

(control_transfer_statement "continue" @cfg.exit.continue)

(control_transfer_statement (throw_keyword) @cfg.exit.throw)
