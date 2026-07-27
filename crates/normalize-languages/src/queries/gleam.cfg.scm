; Gleam CFG query
; Captures control flow nodes for CFG construction.
; See normalize-cfg for the full capture vocabulary.
; Verified against arborium Gleam grammar node types via `normalize syntax ast`.
;
; Gleam is a functional language. `case` (pattern matching) is the only
; control-flow construct — this arborium grammar has no boolean if/else
; expression node: `if` only appears as the anonymous keyword inside a
; `case_clause_guard` (`n if n > 0 -> ...`), and as `target_group`'s
; `@target(erlang) if { ... }` conditional-compilation blocks. A hand-tested
; `if x { ... } else { ... }` snippet parses as two bare identifiers
; followed by anonymous blocks — not a branch node — confirming the
; grammar genuinely doesn't model boolean if/else as control flow, so no
; `@cfg.branch` pattern is emitted here.
; No loops, break, continue. `panic`/`todo` are exit forms.

; ---------------------------------------------------------------------------
; case (match/pattern matching)
; ---------------------------------------------------------------------------

(case
  subjects: (_) @cfg.match.scrutinee
  clauses: (case_clauses
    (case_clause) @cfg.match.arm
  )
) @cfg.match

; ---------------------------------------------------------------------------
; Exits
; ---------------------------------------------------------------------------

; panic and todo are exit expressions in Gleam
(panic) @cfg.exit.throw

(todo) @cfg.exit.throw
