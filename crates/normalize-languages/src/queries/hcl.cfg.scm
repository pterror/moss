; HCL (HashiCorp Configuration Language) CFG query
; Captures control flow nodes for CFG construction.
; See normalize-cfg for the full capture vocabulary.
; Verified against arborium HCL grammar node types via `normalize syntax ast`.
;
; HCL has limited control flow: conditional (ternary) expressions
; and for expressions/comprehensions. No loops or exit statements.
;
; `conditional` has no fields at all — condition/then/else are three
; unfielded `(expression)` children in positional order around the
; anonymous `?`/`:` tokens.

; ---------------------------------------------------------------------------
; conditional (ternary expression — branch)
; ---------------------------------------------------------------------------

(conditional
  (expression) @cfg.branch.condition
  .
  (expression) @cfg.branch.then
  .
  (expression) @cfg.branch.else
) @cfg.branch

; ---------------------------------------------------------------------------
; for (comprehension — loop-like construct)
; ---------------------------------------------------------------------------

(for_tuple_expr
  (for_cond) @cfg.loop.condition
) @cfg.loop

(for_tuple_expr) @cfg.loop

(for_object_expr
  (for_cond) @cfg.loop.condition
) @cfg.loop

(for_object_expr) @cfg.loop
