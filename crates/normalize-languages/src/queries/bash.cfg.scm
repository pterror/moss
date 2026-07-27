; Bash CFG query
; Captures control flow nodes for CFG construction.
; See normalize-cfg for the full capture vocabulary.
; Verified against arborium Bash grammar node types.

; ---------------------------------------------------------------------------
; if / elif / else (branch)
; ---------------------------------------------------------------------------

; if_statement's "then" branch is unfielded (`optional($._terminated_statement)`
; directly after the anonymous "then" token) — no `body:` field exists on this
; node. The anchor below binds to the statement immediately following the
; condition, skipping over the anonymous "then" keyword token.
(if_statement
  condition: (_) @cfg.branch.condition
  .
  (_) @cfg.branch.then
) @cfg.branch

(if_statement
  (else_clause) @cfg.branch.else
) @cfg.branch

(if_statement
  (elif_clause) @cfg.branch.else
) @cfg.branch

; ---------------------------------------------------------------------------
; case (match)
; ---------------------------------------------------------------------------

(case_statement
  value: (_) @cfg.match.scrutinee
  (case_item) @cfg.match.arm
) @cfg.match

; ---------------------------------------------------------------------------
; for (loop)
; ---------------------------------------------------------------------------

(for_statement
  variable: (_) @cfg.loop.condition
  body: (_) @cfg.loop.body
) @cfg.loop

(c_style_for_statement
  condition: (_) @cfg.loop.condition
  body: (_) @cfg.loop.body
) @cfg.loop

(c_style_for_statement
  body: (_) @cfg.loop.body
) @cfg.loop

; ---------------------------------------------------------------------------
; while / until (loop with condition)
; ---------------------------------------------------------------------------

(while_statement
  condition: (_) @cfg.loop.condition
  body: (_) @cfg.loop.body
) @cfg.loop

; ---------------------------------------------------------------------------
; Exits
; ---------------------------------------------------------------------------

; Bash has no dedicated return/break/continue statement node types — they're
; ordinary builtin commands (`command` with `command_name: "return"`, etc).

(command
  name: (command_name) @_cmd
  (#eq? @_cmd "return")
) @cfg.exit.return

(command
  name: (command_name) @_cmd
  (#eq? @_cmd "break")
) @cfg.exit.break

(command
  name: (command_name) @_cmd
  (#eq? @_cmd "continue")
) @cfg.exit.continue
