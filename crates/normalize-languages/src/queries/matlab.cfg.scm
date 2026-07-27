; MATLAB CFG query
; Captures control flow nodes for CFG construction.
; See normalize-cfg for the full capture vocabulary.
; Verified against arborium MATLAB grammar node types and against
; crates/normalize-languages/tests/fixtures/matlab/sample.m via the
; GrammarLoader directly (NOT `normalize syntax ast/query -p <file>` —
; MATLAB's ".m" extension collides with Objective-C's, and the CLI's
; extension-based language detection resolves fixtures/matlab/sample.m
; to the objc grammar; see TODO.md for that pre-existing bug).
;
; if_statement has a "condition" field but no "body" field — the
; then-branch is an unnamed (block) child, with optional unnamed
; (elseif_clause)/(else_clause) siblings. for_statement has no fields
; at all: its unnamed children are (iterator) and (block).

; ---------------------------------------------------------------------------
; if / elseif / else (branch)
; ---------------------------------------------------------------------------

(if_statement
  condition: (_) @cfg.branch.condition
  (block) @cfg.branch.then
  (elseif_clause) @cfg.branch.else
) @cfg.branch

(if_statement
  condition: (_) @cfg.branch.condition
  (block) @cfg.branch.then
  (else_clause) @cfg.branch.else
) @cfg.branch

(if_statement
  condition: (_) @cfg.branch.condition
  (block) @cfg.branch.then
  .
) @cfg.branch

; ---------------------------------------------------------------------------
; switch / case (match)
; ---------------------------------------------------------------------------

(switch_statement
  condition: (_) @cfg.match.scrutinee
  (case_clause) @cfg.match.arm
) @cfg.match

; ---------------------------------------------------------------------------
; for (loop)
; ---------------------------------------------------------------------------

(for_statement
  (iterator) @cfg.loop.condition
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
; try / catch (exception handling)
; ---------------------------------------------------------------------------

(try_statement
  (block) @cfg.try.body
) @cfg.try

(catch_clause) @cfg.try.catch

; ---------------------------------------------------------------------------
; Exits
; ---------------------------------------------------------------------------

(return_statement) @cfg.exit.return

(break_statement) @cfg.exit.break

(continue_statement) @cfg.exit.continue

; error() is the throw equivalent in MATLAB
(function_call
  name: (identifier) @_fn
  (#match? @_fn "^(error|rethrow)$")
) @cfg.exit.throw
