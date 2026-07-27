; Erlang CFG query
; Captures control flow nodes for CFG construction.
; See normalize-cfg for the full capture vocabulary.
; Verified against arborium Erlang grammar node types via `normalize syntax ast`.
;
; Erlang uses if_expr/case_expr/receive_expr/try_expr for control flow.
; Pattern matching in function clauses is the primary branching mechanism.
; No imperative break/continue — tail recursion replaces loops.

; ---------------------------------------------------------------------------
; if (branch — guards as conditions)
; ---------------------------------------------------------------------------

; `if_expr` clauses are flat siblings (`clauses:` field, repeated), each with
; its own `guard:`/`body:` — mirrors how flat elsif-style chains are handled
; elsewhere in this codebase (e.g. Elm): each clause is its own branch.
(if_clause
  guard: (_) @cfg.branch.condition
  body: (_) @cfg.branch.then
) @cfg.branch

; ---------------------------------------------------------------------------
; case (match)
; ---------------------------------------------------------------------------

(case_expr
  expr: (_) @cfg.match.scrutinee
  clauses: (cr_clause) @cfg.match.arm
) @cfg.match

; ---------------------------------------------------------------------------
; receive (message-passing — treated as match; `after` timeout is an
; additional arm reached when no message matches within the deadline)
; ---------------------------------------------------------------------------

(receive_expr
  clauses: (cr_clause) @cfg.match.arm
  after: (receive_after
    body: (_) @cfg.match.arm
  )?
) @cfg.match

; ---------------------------------------------------------------------------
; try / catch / after
; ---------------------------------------------------------------------------

(try_expr
  exprs: (_) @cfg.try.body
  catch: (catch_clause
    class: (try_class
      class: (atom) @cfg.try.catch.type)
  ) @cfg.try.catch
  after: (try_after
    exprs: (_) @cfg.try.finally
  )?
) @cfg.try

; ---------------------------------------------------------------------------
; Exits
; ---------------------------------------------------------------------------

; throw/exit/error are built-in calls in the `erlang` module
(call
  expr: (remote
    module: (remote_module
      module: (atom) @_m)
    fun: (atom) @_fn)
  (#eq? @_m "erlang")
  (#match? @_fn "^(throw|exit|error)$")
) @cfg.exit.throw
