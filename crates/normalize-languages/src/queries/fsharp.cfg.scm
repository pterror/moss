; F# CFG query
; Captures control flow nodes for CFG construction.
; See normalize-cfg for the full capture vocabulary.
; Verified against arborium F# grammar node types via `normalize syntax ast`.
;
; F# is a functional-first language. Control flow includes if_expression,
; match_expression, for/while loops, and try expressions. There is no
; dedicated `return_expression`/`raise_expression` node type — `raise`,
; `reraise`, and `failwith`/`failwithf` are ordinary function applications,
; and explicit `return`/`yield` (used inside computation expressions like
; `async { }`/`seq { }`) are anonymous keyword tokens inside
; `prefixed_expression`. Loop/branch bodies are unfielded siblings —
; anonymous keyword tokens (`in`, `do`, `with`, `finally`) anchor the
; surrounding condition/body captures.

; ---------------------------------------------------------------------------
; if / elif / else (branch expression)
; ---------------------------------------------------------------------------

(if_expression
  guard: (_) @cfg.branch.condition
  then: (_) @cfg.branch.then
  else: (_)? @cfg.branch.else
) @cfg.branch

(elif_expression
  guard: (_) @cfg.branch.condition
  then: (_) @cfg.branch.then
) @cfg.branch

; ---------------------------------------------------------------------------
; match (pattern matching — rules as arms)
; ---------------------------------------------------------------------------

(match_expression
  (_) @cfg.match.scrutinee
  .
  "with"
  .
  (rules
    (rule) @cfg.match.arm
  )
) @cfg.match

; ---------------------------------------------------------------------------
; for / for-each (loop)
; ---------------------------------------------------------------------------

(for_expression
  "in"
  .
  (_) @cfg.loop.condition
  .
  "do"
  .
  (_) @cfg.loop.body
) @cfg.loop

; ---------------------------------------------------------------------------
; while (loop with condition)
; ---------------------------------------------------------------------------

(while_expression
  "while"
  .
  (_) @cfg.loop.condition
  .
  "do"
  .
  (_) @cfg.loop.body
) @cfg.loop

; ---------------------------------------------------------------------------
; try / with / finally
; ---------------------------------------------------------------------------

(try_expression
  "try"
  .
  (_) @cfg.try.body
  "with"
  (rules
    (rule) @cfg.try.catch
  )
) @cfg.try

(try_expression
  "try"
  .
  (_) @cfg.try.body
  "finally"
  .
  (_) @cfg.try.finally
) @cfg.try

; ---------------------------------------------------------------------------
; Exits
; ---------------------------------------------------------------------------

; `return`/`return!` inside a computation expression (async/seq/task/...)
(prefixed_expression "return") @cfg.exit.return

; `raise`, `reraise ()`, `failwith`/`failwithf` are plain function calls
(application_expression
  .
  (long_identifier_or_op
    (identifier) @_fn)
  (#match? @_fn "^(raise|reraise|failwith|failwithf)$")
) @cfg.exit.throw

; ---------------------------------------------------------------------------
; Effects
; ---------------------------------------------------------------------------

; `yield`/`yield!` inside a computation expression (seq/task/...)
(prefixed_expression "yield") @cfg.effect.yield
