; Groovy CFG query
; Captures control flow nodes for CFG construction.
; See normalize-cfg for the full capture vocabulary.
; Verified against arborium Groovy grammar node types via `normalize syntax ast`.
;
; This grammar has no dedicated `throw_statement` node — `throw expr` parses
; as a `juxt_function_call` whose `function:` identifier happens to be the
; text "throw", matched below via predicate. `try_statement` has no
; `catch_clause`/`finally_clause` wrapper nodes either — `catch_body:` and
; `finally_body:` are flat optional fields directly on `try_statement`.
; `for_loop`'s condition lives one level deeper, inside `for_parameters`.

; ---------------------------------------------------------------------------
; if / else (branch)
; ---------------------------------------------------------------------------

(if_statement
  condition: (_) @cfg.branch.condition
  body: (_) @cfg.branch.then
  else_body: (_) @cfg.branch.else
) @cfg.branch

(if_statement
  condition: (_) @cfg.branch.condition
  body: (_) @cfg.branch.then
  .
  ; no alternative
) @cfg.branch

; ---------------------------------------------------------------------------
; switch (match)
; ---------------------------------------------------------------------------

(switch_statement
  value: (_) @cfg.match.scrutinee
  body: (switch_block
    (case) @cfg.match.arm
  )
) @cfg.match

; ---------------------------------------------------------------------------
; for / for-in (loop)
; ---------------------------------------------------------------------------

(for_loop
  (for_parameters
    condition: (_) @cfg.loop.condition
  )
  body: (_) @cfg.loop.body
) @cfg.loop

(for_in_loop
  collection: (_) @cfg.loop.condition
  body: (_) @cfg.loop.body
) @cfg.loop

; ---------------------------------------------------------------------------
; while (loop with condition)
; ---------------------------------------------------------------------------

(while_loop
  condition: (_) @cfg.loop.condition
  body: (_) @cfg.loop.body
) @cfg.loop

; ---------------------------------------------------------------------------
; try / catch / finally
; ---------------------------------------------------------------------------

(try_statement
  body: (_) @cfg.try.body
  catch_body: (_)? @cfg.try.catch
  finally_body: (_)? @cfg.try.finally
) @cfg.try

; ---------------------------------------------------------------------------
; Exits
; ---------------------------------------------------------------------------

(return) @cfg.exit.return

(break) @cfg.exit.break

(continue) @cfg.exit.continue

; `throw expr` is not a distinct grammar construct — it parses as a
; `juxt_function_call` with function identifier text "throw".
(juxt_function_call
  function: (identifier) @_fn
  (#eq? @_fn "throw")
) @cfg.exit.throw
