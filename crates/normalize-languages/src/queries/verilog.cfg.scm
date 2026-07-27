; Verilog/SystemVerilog CFG query
; Captures control flow nodes for CFG construction.
; See normalize-cfg for the full capture vocabulary.
; Verified against arborium Verilog grammar node types using real fixtures.
; Note: `cond_predicate`, `case_expression`, `statement_or_null`, and
; `for_initialization`/`for_step` are named node types, NOT fields — they
; appear as bare children, not `field: (...)`. `if_statement` has no
; `else_clause` wrapper: `else` is an anonymous token sibling followed
; directly by another `statement_or_null`. `return`/`break`/`continue` are
; not their own statement node types — they are anonymous tokens inside a
; shared `jump_statement` node (`disable` is its own `disable_statement`).

; ---------------------------------------------------------------------------
; if / else (branch — conditional_statement)
; ---------------------------------------------------------------------------

(conditional_statement
  (cond_predicate) @cfg.branch.condition
  (statement_or_null) @cfg.branch.then
  "else"
  (statement_or_null) @cfg.branch.else
) @cfg.branch

(conditional_statement
  (cond_predicate) @cfg.branch.condition
  (statement_or_null) @cfg.branch.then
  .
) @cfg.branch

; ---------------------------------------------------------------------------
; case / casez / casex (match)
; ---------------------------------------------------------------------------

(case_statement
  (case_expression) @cfg.match.scrutinee
  (case_item) @cfg.match.arm
) @cfg.match

; ---------------------------------------------------------------------------
; for / while / forever / repeat (loop_statement)
; ---------------------------------------------------------------------------

(loop_statement
  (for_initialization) @cfg.loop.condition
  (for_step) @cfg.loop.condition
  (statement_or_null) @cfg.loop.body
) @cfg.loop

(loop_statement
  (statement_or_null) @cfg.loop.body
) @cfg.loop

; ---------------------------------------------------------------------------
; Exits
; ---------------------------------------------------------------------------

(jump_statement "return" @cfg.exit.return)

(jump_statement "break" @cfg.exit.break)

(jump_statement "continue" @cfg.exit.continue)

(disable_statement) @cfg.exit.throw
