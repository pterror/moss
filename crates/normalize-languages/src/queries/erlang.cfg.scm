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

; `clauses:` is optional (`receive_expr.clauses` is a non-required,
; repeatable field per node-types.json) — `receive after 0 -> ok end`, a
; common non-blocking "drain the mailbox" idiom, has no message clauses at
; all. Verified via `normalize syntax query`: without `?` here the whole
; `@cfg.match` silently failed to match that idiom.
(receive_expr
  clauses: (cr_clause)? @cfg.match.arm
  after: (receive_after
    body: (_) @cfg.match.arm
  )?
) @cfg.match

; ---------------------------------------------------------------------------
; try / catch / after
; ---------------------------------------------------------------------------

; All three of `clauses:` (the `of` pattern-match arms), `catch:`, and
; `after:` are independently optional on `try_expr` per node-types.json —
; `try X of P -> R after C end` (no catch), `try X of P -> R catch ... end`
; (of + catch, no after), and `try X catch Pattern -> R end` (no class:,
; the implicit-`throw`-class bare-pattern catch form) are all real,
; verified-parsing forms. The previous non-optional `catch:` + hardcoded
; `class: (atom)` meant `@cfg.try`/`@cfg.try.catch` failed to match ANY of
; them — only the rare `error:Reason`/`throw:Value` literal-atom-class form
; (which is what the pre-existing sample.erl fixture happened to use)
; matched, hiding the bug. `try_class.class` is the `_name` supertype,
; which allows `var` (the far more common `Class:Reason` /
; `_:Reason`-catch-all form used with modern `?LOG_ERROR`-style error
; reporting) in addition to `atom`; `[(atom) (var)]` covers both, and the
; `class:` field itself is optional (the bare-pattern catch form).
(try_expr
  exprs: (_) @cfg.try.body
  clauses: (cr_clause)? @cfg.match.arm
  catch: (catch_clause
    class: (try_class
      class: [(atom) (var)] @cfg.try.catch.type)?
  )? @cfg.try.catch
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

; throw/exit/error are also auto-imported BIFs, callable bare (without the
; `erlang:` prefix) — this is by far the more common real-world form.
; Verified via `normalize syntax query`: the explicit-prefix pattern above
; matched 0 occurrences of bare `throw(Reason)`/`exit(normal)`/
; `error(badarg)`, which is what the vast majority of real Erlang code
; actually writes.
(call
  expr: (atom) @_fn
  (#match? @_fn "^(throw|exit|error)$")
) @cfg.exit.throw
