; Awk CFG query
; Captures control flow nodes for CFG construction.
; See normalize-cfg for the full capture vocabulary.
; Verified against arborium Awk grammar node types.

; ---------------------------------------------------------------------------
; if / else (branch)
; ---------------------------------------------------------------------------

; awk's if/while/for/do-while bodies are *unfielded* — the grammar allows
; `choice($.block, $._statement, ';')` with no field name, so a bare
; statement body can't be distinguished from a following `else_clause` or
; stray `comment` node via a field lookup. We match the `(block)` form
; (the idiomatic, and only form used in real-world awk scripts in our
; fixtures) explicitly by type instead.

(if_statement
  condition: (_) @cfg.branch.condition
  (block) @cfg.branch.then
) @cfg.branch

(if_statement
  condition: (_) @cfg.branch.condition
  (else_clause) @cfg.branch.else
) @cfg.branch

; ---------------------------------------------------------------------------
; for / for-in (loop)
; ---------------------------------------------------------------------------

(for_statement
  condition: (_) @cfg.loop.condition
  (block) @cfg.loop.body
) @cfg.loop

(for_in_statement
  left: (_) @cfg.loop.condition
  right: (_) @cfg.loop.condition
  (block) @cfg.loop.body
) @cfg.loop

; ---------------------------------------------------------------------------
; while (loop with condition)
; ---------------------------------------------------------------------------

(while_statement
  condition: (_) @cfg.loop.condition
  (block) @cfg.loop.body
) @cfg.loop

; ---------------------------------------------------------------------------
; do-while (loop with condition at end)
; ---------------------------------------------------------------------------

(do_while_statement
  (block) @cfg.loop.body
  condition: (_) @cfg.loop.condition
) @cfg.loop

; ---------------------------------------------------------------------------
; Exits
; ---------------------------------------------------------------------------

(return_statement) @cfg.exit.return

(break_statement) @cfg.exit.break

(continue_statement) @cfg.exit.continue

(next_statement) @cfg.exit.continue

(exit_statement) @cfg.exit.throw
