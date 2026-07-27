; PHP CFG query
; Captures control flow nodes for CFG construction.
; See normalize-cfg for the full capture vocabulary.
; Verified against arborium PHP grammar via
;   normalize syntax ast <file> --compact --depth=-1
;   normalize syntax query <query> -p <file> --show-source
;
; There is no "throw_statement" node — throw is only ever
; throw_expression (usable in expression position since PHP 8.0).
; match_expression's arms nest match_condition_list one level deeper
; than expected (inside match_conditional_expression/
; match_default_expression, not a direct sibling). foreach_statement
; has no field names on its header at all — the iterated collection is
; whichever child directly precedes the literal "as" keyword,
; regardless of whether the loop binds `$v` or `$k => $v`.

; ---------------------------------------------------------------------------
; if / else (branch)
; ---------------------------------------------------------------------------

(if_statement
  condition: (_) @cfg.branch.condition
  body: (_) @cfg.branch.then
  (else_if_clause
    condition: (_)
    body: (_)) @cfg.branch.else
) @cfg.branch

(if_statement
  condition: (_) @cfg.branch.condition
  body: (_) @cfg.branch.then
  (else_clause
    body: (_) @cfg.branch.else)
) @cfg.branch

(if_statement
  condition: (_) @cfg.branch.condition
  body: (_) @cfg.branch.then
  .
) @cfg.branch

; ---------------------------------------------------------------------------
; match (PHP 8+)
; ---------------------------------------------------------------------------

; Each match arm is (match_conditional_expression conditional_expressions:
; (match_condition_list) ...) or (match_default_expression) for `default =>`
; — match_condition_list is nested inside the arm node, not a sibling of
; match_expression directly.
(match_expression
  condition: (_) @cfg.match.scrutinee
  body: (match_block
    (match_conditional_expression) @cfg.match.arm
  )
) @cfg.match

(match_expression
  condition: (_) @cfg.match.scrutinee
  body: (match_block
    (match_default_expression) @cfg.match.arm
  )
) @cfg.match

; ---------------------------------------------------------------------------
; switch (match)
; ---------------------------------------------------------------------------

(switch_statement
  condition: (_) @cfg.match.scrutinee
  body: (switch_block
    (case_statement) @cfg.match.arm
  )
) @cfg.match

(switch_statement
  condition: (_) @cfg.match.scrutinee
  body: (switch_block
    (default_statement) @cfg.match.arm
  )
) @cfg.match

; ---------------------------------------------------------------------------
; for (C-style loop)
; ---------------------------------------------------------------------------

(for_statement
  condition: (_) @cfg.loop.condition
  body: (_) @cfg.loop.body
) @cfg.loop

(for_statement
  body: (_) @cfg.loop.body
) @cfg.loop

; ---------------------------------------------------------------------------
; foreach (loop over collection)
; ---------------------------------------------------------------------------

; foreach_statement has no field names on the header — key/value/pair
; children are positional. The iterated collection is always the child
; immediately followed by the literal "as" keyword (whether the loop
; captures `$v` or `$k => $v`, verified via a probe with both forms).
(foreach_statement
  (_) @cfg.loop.condition
  "as"
  body: (_) @cfg.loop.body
) @cfg.loop

; ---------------------------------------------------------------------------
; while (loop with condition)
; ---------------------------------------------------------------------------

(while_statement
  condition: (_) @cfg.loop.condition
  body: (_) @cfg.loop.body
) @cfg.loop

; ---------------------------------------------------------------------------
; do-while (loop with condition at end)
; ---------------------------------------------------------------------------

(do_statement
  body: (_) @cfg.loop.body
  condition: (_) @cfg.loop.condition
) @cfg.loop

; ---------------------------------------------------------------------------
; try / catch / finally
; ---------------------------------------------------------------------------

(try_statement
  body: (_) @cfg.try.body
) @cfg.try

(catch_clause) @cfg.try.catch

(finally_clause) @cfg.try.finally

; ---------------------------------------------------------------------------
; Exits
; ---------------------------------------------------------------------------

(return_statement) @cfg.exit.return

(break_statement) @cfg.exit.break

(continue_statement) @cfg.exit.continue

; PHP models throw as an expression (usable in expression position since
; PHP 8.0), not a statement — there is no "throw_statement" node type.
(throw_expression) @cfg.exit.throw
