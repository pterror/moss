; Objective-C CFG query
; Captures control flow nodes for CFG construction.
; See normalize-cfg for the full capture vocabulary.
; Verified against arborium Objective-C grammar node types and against
; crates/normalize-languages/tests/fixtures/objc/sample.m via
;   normalize syntax query <query> -p <file> --show-source
; Shares C's grammar base with @try/@catch/@finally additions.
;
; try_statement/catch_clause have NO fields — their (compound_statement)
; body is an unnamed child, matched positionally.
;
; Fast enumeration ("for (Type *x in collection) { ... }") has no
; dedicated node type in this grammar (no "for_in_statement" — that
; was a fabricated guess). Probing it directly shows the parser
; accepts the syntax but does NOT populate for_statement's normal
; initializer/condition/update/body fields for it — the type
; specifier, declarator, collection expression and body all come back
; as flat, field-less children. This construct is not reliably
; distinguishable from a malformed C for-loop in this grammar, so it
; is left uncaptured rather than fabricating a pattern that would
; either never match or match nonsense.

; ---------------------------------------------------------------------------
; if / else (branch)
; ---------------------------------------------------------------------------

(if_statement
  condition: (_) @cfg.branch.condition
  consequence: (_) @cfg.branch.then
  alternative: (_) @cfg.branch.else
) @cfg.branch

(if_statement
  condition: (_) @cfg.branch.condition
  consequence: (_) @cfg.branch.then
  .
) @cfg.branch

; ---------------------------------------------------------------------------
; switch (match)
; ---------------------------------------------------------------------------

(switch_statement
  condition: (_) @cfg.match.scrutinee
  body: (compound_statement
    (case_statement) @cfg.match.arm
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
; @try / @catch / @finally (Objective-C exception handling)
; ---------------------------------------------------------------------------

(try_statement
  (compound_statement) @cfg.try.body
) @cfg.try

(catch_clause) @cfg.try.catch

(finally_clause) @cfg.try.finally

; ---------------------------------------------------------------------------
; Exits
; ---------------------------------------------------------------------------

(return_statement) @cfg.exit.return

(break_statement) @cfg.exit.break

(continue_statement) @cfg.exit.continue

(throw_statement) @cfg.exit.throw

(goto_statement) @cfg.exit.throw
