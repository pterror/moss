; Starlark (Bazel build language) CFG query
; Captures control flow nodes for CFG construction.
; See normalize-cfg for the full capture vocabulary.
;
; Verified against arborium Starlark grammar via
;   normalize syntax ast <file> --compact --depth=-1
;   normalize syntax query <query> -p <file> --show-source
;
; Starlark is Python-like. if_statement's then-arm field is
; "consequence" (not "body" — that name is used elsewhere, e.g.
; function_definition's block). Control flow: if/for statements,
; conditional expressions. return/break/continue as exits.

; ---------------------------------------------------------------------------
; if / elif / else (branch)
; ---------------------------------------------------------------------------

(if_statement
  condition: (_) @cfg.branch.condition
  consequence: (_) @cfg.branch.then
  (elif_clause) @cfg.branch.else
) @cfg.branch

(if_statement
  condition: (_) @cfg.branch.condition
  consequence: (_) @cfg.branch.then
  (else_clause
    body: (_) @cfg.branch.else)
) @cfg.branch

(if_statement
  condition: (_) @cfg.branch.condition
  consequence: (_) @cfg.branch.then
  .
) @cfg.branch

(elif_clause
  condition: (_) @cfg.branch.condition
  consequence: (_) @cfg.branch.then
) @cfg.branch

; ---------------------------------------------------------------------------
; for (loop)
; ---------------------------------------------------------------------------

(for_statement
  left: (_) @cfg.loop.condition
  right: (_)
  body: (_) @cfg.loop.body
) @cfg.loop

; ---------------------------------------------------------------------------
; Exits
; ---------------------------------------------------------------------------

(return_statement) @cfg.exit.return

(break_statement) @cfg.exit.break

(continue_statement) @cfg.exit.continue
