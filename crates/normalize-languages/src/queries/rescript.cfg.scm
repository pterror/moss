; ReScript CFG query
; Captures control flow nodes for CFG construction.
; See normalize-cfg for the full capture vocabulary.
;
; Verified against arborium ReScript grammar via
;   normalize syntax ast <file> --compact --depth=-1
;   normalize syntax query <query> -p <file> --show-source
;
; ReScript is a functional language compiling to JavaScript: functions
; return the value of their last expression implicitly — there is no
; explicit return keyword and no "return_expression" node type, so no
; cfg.exit.return capture exists here (nothing to capture).
;
; if_expression has NO "condition"/"consequence"/"alternative" fields
; at all — the condition and (block) body are unnamed positional
; children, and else_if_clause/else_clause are flat unnamed siblings
; of the initial if/block pair (not nested inside each other), mirrored
; here the same way Lua's elseif_statement/else_statement are handled.
; switch_expression's scrutinee is likewise an unnamed positional child
; (no "value" field) — anchored with a leading "." so the wildcard
; can't also bind to a later (switch_match) arm.

; ---------------------------------------------------------------------------
; if / else if / else (branch expression)
; ---------------------------------------------------------------------------

(if_expression
  (_) @cfg.branch.condition
  (block) @cfg.branch.then
  (else_if_clause) @cfg.branch.else
) @cfg.branch

(if_expression
  (_) @cfg.branch.condition
  (block) @cfg.branch.then
  (else_clause) @cfg.branch.else
) @cfg.branch

(if_expression
  (_) @cfg.branch.condition
  (block) @cfg.branch.then
  .
) @cfg.branch

(else_if_clause
  (_) @cfg.branch.condition
  (block) @cfg.branch.then
) @cfg.branch

(else_clause
  (block) @cfg.branch.then
) @cfg.branch

; ---------------------------------------------------------------------------
; switch (match/pattern matching)
; ---------------------------------------------------------------------------

(switch_expression
  . (_) @cfg.match.scrutinee
  (switch_match) @cfg.match.arm
) @cfg.match

; ---------------------------------------------------------------------------
; Exits
; ---------------------------------------------------------------------------

; raise is the throw equivalent
(call_expression
  function: (value_identifier) @_fn
  (#match? @_fn "^(raise|failwith|invalid_arg)$")
) @cfg.exit.throw
