; VHDL CFG query
; Captures control flow nodes for CFG construction.
; See normalize-cfg for the full capture vocabulary.
; Verified against arborium VHDL grammar node types using real fixtures.
; Note: `if_statement` has no top-level `condition:` field and no
; `elsif_sequence_of_statements`/`else_sequence_of_statements` wrapper nodes
; (those don't exist in the grammar). Instead it has repeatable named child
; nodes `(if ...)`, `(elsif ...)` (0+), `(else ...)` — each wrapping its own
; `conditional_expression` + `sequence_of_statements`. `case_statement`'s
; scrutinee is a bare `(expression)` child, not an `expression:` field.
; `loop_statement` wraps `(for_loop)` or `(while_loop)` as a bare child
; (there is no `iteration_scheme` node), followed by a bare
; `(sequence_of_statements)` body.

; ---------------------------------------------------------------------------
; if / elsif / else (branch)
; ---------------------------------------------------------------------------

(if_statement
  (if
    (conditional_expression) @cfg.branch.condition
    (sequence_of_statements) @cfg.branch.then)
) @cfg.branch

(if_statement
  (elsif
    (conditional_expression) @cfg.branch.condition
    (sequence_of_statements) @cfg.branch.then) @cfg.branch.else
) @cfg.branch

(if_statement
  (else
    (sequence_of_statements) @cfg.branch.else)
) @cfg.branch

; ---------------------------------------------------------------------------
; case (match)
; ---------------------------------------------------------------------------

(case_statement
  (expression) @cfg.match.scrutinee
  (case_statement_alternative) @cfg.match.arm
) @cfg.match

; ---------------------------------------------------------------------------
; loop (for / while / bare loop)
; ---------------------------------------------------------------------------

(loop_statement
  (for_loop) @cfg.loop.condition
  (sequence_of_statements) @cfg.loop.body
) @cfg.loop

(loop_statement
  (while_loop) @cfg.loop.condition
  (sequence_of_statements) @cfg.loop.body
) @cfg.loop

(loop_statement
  (sequence_of_statements) @cfg.loop.body
) @cfg.loop

; ---------------------------------------------------------------------------
; Exits
; ---------------------------------------------------------------------------

(return_statement) @cfg.exit.return

(exit_statement) @cfg.exit.break

(next_statement) @cfg.exit.continue
