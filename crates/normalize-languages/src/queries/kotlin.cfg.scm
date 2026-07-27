; Kotlin CFG query
; Captures control flow nodes for CFG construction.
; See normalize-cfg for the full capture vocabulary.
; Verified against arborium Kotlin grammar node types and against
; crates/normalize-languages/tests/fixtures/kotlin/sample.kt via
;   normalize syntax query <query> -p <file> --show-source
;
; Kotlin uses if_expression (has condition/consequence/alternative
; fields, all "control_structure_body") and when_expression for
; conditional branching. for_statement / while_statement /
; do_while_statement have NO named fields at all — children (loop
; variable, iterable, (control_structure_body) body) are flat and
; unnamed; there is no "loop_range" node type. return/break/continue/
; throw are not separate node types either — they are all
; (jump_expression) wrapping a literal keyword token as its first
; child, matched here via anonymous-token patterns.

; ---------------------------------------------------------------------------
; if / else (branch) — expression in Kotlin
; ---------------------------------------------------------------------------

(if_expression
  condition: (_) @cfg.branch.condition
  consequence: (_) @cfg.branch.then
  alternative: (_) @cfg.branch.else
) @cfg.branch

(if_expression
  condition: (_) @cfg.branch.condition
  consequence: (_) @cfg.branch.then
  .
  ; no alternative
) @cfg.branch

; ---------------------------------------------------------------------------
; when (match) — replaces switch in Kotlin
; ---------------------------------------------------------------------------

(when_expression
  (when_subject) @cfg.match.scrutinee
  (when_entry) @cfg.match.arm
) @cfg.match

; when without subject (used as if-else chain)
(when_expression
  (when_entry) @cfg.match.arm
) @cfg.match

; ---------------------------------------------------------------------------
; for (loop over collection)
; ---------------------------------------------------------------------------

(for_statement
  (variable_declaration) @cfg.loop.condition
  (control_structure_body) @cfg.loop.body
) @cfg.loop

; ---------------------------------------------------------------------------
; while (loop with condition)
; ---------------------------------------------------------------------------

(while_statement
  (control_structure_body) @cfg.loop.body
) @cfg.loop

; ---------------------------------------------------------------------------
; do-while (loop with condition at end)
; ---------------------------------------------------------------------------

(do_while_statement
  (control_structure_body) @cfg.loop.body
) @cfg.loop

; ---------------------------------------------------------------------------
; try / catch / finally
; ---------------------------------------------------------------------------

(try_expression
  (statements) @cfg.try.body
) @cfg.try

(catch_block) @cfg.try.catch

(finally_block) @cfg.try.finally

; ---------------------------------------------------------------------------
; Exits — all are (jump_expression <keyword>), no dedicated node types
; ---------------------------------------------------------------------------

(jump_expression "return") @cfg.exit.return

(jump_expression "break") @cfg.exit.break

(jump_expression "continue") @cfg.exit.continue

(jump_expression "throw") @cfg.exit.throw
