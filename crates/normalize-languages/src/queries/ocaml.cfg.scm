; OCaml CFG query
; Captures control flow nodes for CFG construction.
; See normalize-cfg for the full capture vocabulary.
; Verified against arborium OCaml grammar node types and against
; crates/normalize-languages/tests/fixtures/ocaml/sample.ml (plus a
; scratch snippet exercising for/while/try, which the fixture lacks)
; via normalize syntax query <query> -p <file> --show-source.
;
; OCaml is expression-oriented. if_expression / while_expression have
; a "condition" field but NO "consequence" field — then_clause /
; else_clause / do_clause are unnamed children. for_expression has
; "name"/"from"/"to" fields and an unnamed (do_clause) body.
; match_expression has an "expression" field (scrutinee) and unnamed
; (match_case) arms. try_expression's "expression" field is the
; *protected body*, not the scrutinee, with (match_case) arms as the
; catch handlers.

; ---------------------------------------------------------------------------
; if / else (branch expression)
; ---------------------------------------------------------------------------

(if_expression
  condition: (_) @cfg.branch.condition
  (then_clause) @cfg.branch.then
  (else_clause) @cfg.branch.else
) @cfg.branch

(if_expression
  condition: (_) @cfg.branch.condition
  (then_clause) @cfg.branch.then
  .
  ; no else branch
) @cfg.branch

; ---------------------------------------------------------------------------
; match (pattern matching)
; ---------------------------------------------------------------------------

(match_expression
  expression: (_) @cfg.match.scrutinee
  (match_case) @cfg.match.arm
) @cfg.match

; ---------------------------------------------------------------------------
; for (counted loop)
; ---------------------------------------------------------------------------

(for_expression
  from: (_) @cfg.loop.condition
  to: (_)
  (do_clause) @cfg.loop.body
) @cfg.loop

; ---------------------------------------------------------------------------
; while (loop with condition)
; ---------------------------------------------------------------------------

(while_expression
  condition: (_) @cfg.loop.condition
  (do_clause) @cfg.loop.body
) @cfg.loop

; ---------------------------------------------------------------------------
; try / with (exception handling)
; ---------------------------------------------------------------------------

(try_expression
  expression: (_) @cfg.try.body
  (match_case) @cfg.try.catch
) @cfg.try

; ---------------------------------------------------------------------------
; Exits
; ---------------------------------------------------------------------------

; raise is the throw equivalent in OCaml
(application_expression
  function: (value_path (value_name) @_fn)
  (#eq? @_fn "raise")
) @cfg.exit.throw
