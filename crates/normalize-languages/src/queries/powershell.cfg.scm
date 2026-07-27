; PowerShell CFG query
; Captures control flow nodes for CFG construction.
; See normalize-cfg for the full capture vocabulary.
;
; Verified against arborium PowerShell grammar via
;   normalize syntax ast <file> --compact --depth=-1
;   normalize syntax query <query> -p <file> --show-source
;
; if_statement's then-arm is an unnamed (statement_block) child (no
; "body" field); elseif arms are wrapped in a flat elseif_clauses list
; (field elseif_clauses), and else_clause is its own field — neither
; nests inside the other. switch/for/foreach/while/do have no "body"
; field either — bodies are unnamed positional (statement_block)
; children throughout this grammar. return/break/continue/throw are
; NOT their own node types — they are all (flow_control_statement
; <keyword>), matched here via literal keyword tokens (throw also
; carries a trailing pipeline as its thrown value).

; ---------------------------------------------------------------------------
; if / elseif / else (branch)
; ---------------------------------------------------------------------------

(if_statement
  condition: (_) @cfg.branch.condition
  (statement_block) @cfg.branch.then
  elseif_clauses: (elseif_clauses) @cfg.branch.else
) @cfg.branch

(if_statement
  condition: (_) @cfg.branch.condition
  (statement_block) @cfg.branch.then
  else_clause: (else_clause) @cfg.branch.else
) @cfg.branch

(if_statement
  condition: (_) @cfg.branch.condition
  (statement_block) @cfg.branch.then
  .
) @cfg.branch

(elseif_clause
  condition: (_) @cfg.branch.condition
  (statement_block) @cfg.branch.then
) @cfg.branch

(else_clause
  (statement_block) @cfg.branch.then
) @cfg.branch

; ---------------------------------------------------------------------------
; switch (match)
; ---------------------------------------------------------------------------

(switch_statement
  (switch_condition (_) @cfg.match.scrutinee)
  (switch_body
    (switch_clauses
      (switch_clause) @cfg.match.arm))
) @cfg.match

; ---------------------------------------------------------------------------
; for / foreach (loop)
; ---------------------------------------------------------------------------

(for_statement
  for_condition: (for_condition) @cfg.loop.condition
  (statement_block) @cfg.loop.body
) @cfg.loop

; foreach_statement has no field names — the iterated collection is
; whatever directly follows the literal "in" keyword.
(foreach_statement
  "in"
  (_) @cfg.loop.condition
  (statement_block) @cfg.loop.body
) @cfg.loop

; ---------------------------------------------------------------------------
; while / do-while (loop)
; ---------------------------------------------------------------------------

(while_statement
  condition: (while_condition) @cfg.loop.condition
  (statement_block) @cfg.loop.body
) @cfg.loop

(do_statement
  (statement_block) @cfg.loop.body
  condition: (while_condition) @cfg.loop.condition
) @cfg.loop

; ---------------------------------------------------------------------------
; try / catch / finally
; ---------------------------------------------------------------------------

(try_statement
  (statement_block) @cfg.try.body
) @cfg.try

(catch_clause) @cfg.try.catch

(finally_clause) @cfg.try.finally

; ---------------------------------------------------------------------------
; Exits
; ---------------------------------------------------------------------------

(flow_control_statement "return") @cfg.exit.return

(flow_control_statement "break") @cfg.exit.break

(flow_control_statement "continue") @cfg.exit.continue

(flow_control_statement "throw") @cfg.exit.throw
