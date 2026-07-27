; Idris CFG query
; Captures control flow nodes for CFG construction.
; See normalize-cfg for the full capture vocabulary.
; Verified against arborium Idris grammar node types and against
; crates/normalize-languages/tests/fixtures/idris/sample.idr via
;   normalize syntax ast <file>
;   normalize syntax query <query> -p <file> --show-source
;
; Idris is a dependently-typed functional language.
; Control flow: if-then-else, case expressions.
; No loops or imperative exits (no break/continue/return/throw nodes).
;
; exp_if has fields "if" (multiple, flat condition tokens — no wrapper
; node), "then" (single exp_then wrapper) and "else" (single exp_else
; wrapper). exp_case has field "condition" (multiple, flat scrutinee
; tokens) and an unnamed "alts" wrapper child containing (alt) nodes —
; there is no bare "exp" node type in this grammar.

; ---------------------------------------------------------------------------
; if / else (branch expression — exp_if)
; ---------------------------------------------------------------------------

(exp_if
  if: (_)+ @cfg.branch.condition
  then: (exp_then) @cfg.branch.then
  else: (exp_else) @cfg.branch.else
) @cfg.branch

; ---------------------------------------------------------------------------
; case (match/pattern matching — exp_case)
; ---------------------------------------------------------------------------

(exp_case
  condition: (_)+ @cfg.match.scrutinee
  (alts (alt) @cfg.match.arm)
) @cfg.match
