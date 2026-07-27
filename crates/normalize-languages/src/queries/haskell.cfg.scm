; Haskell CFG query
; Captures control flow nodes for CFG construction.
; See normalize-cfg for the full capture vocabulary.
; Verified against arborium Haskell grammar node types via `normalize syntax ast`.
;
; Haskell is purely functional — no loops or early exits in the imperative
; sense. Control flow comes from conditional expressions (if-then-else),
; case expressions, and guards. There is no return/break/continue/throw
; in the tree-sitter grammar (these are monadic operations, not AST nodes).
;
; `conditional` uses `if:`/`then:`/`else:` fields (not condition/consequence/
; alternative). `case`'s scrutinee is an unfielded child between the `case`
; and `of` keyword tokens; its arms live under `alternatives: (alternatives
; (alternative ...))`, not bare `(match)` nodes directly.

; ---------------------------------------------------------------------------
; if / then / else (branch expression)
; ---------------------------------------------------------------------------

(conditional
  if: (_) @cfg.branch.condition
  then: (_) @cfg.branch.then
  else: (_) @cfg.branch.else
) @cfg.branch

; ---------------------------------------------------------------------------
; case / match (pattern matching)
; ---------------------------------------------------------------------------

(case
  "case"
  .
  (_) @cfg.match.scrutinee
  .
  "of"
  alternatives: (alternatives
    (alternative) @cfg.match.arm
  )
) @cfg.match

; ---------------------------------------------------------------------------
; guard (boolean guards on function equations / case arms)
; ---------------------------------------------------------------------------

; Guards are captured as branch arms — no body capture, guards are the condition
(guard) @cfg.branch
