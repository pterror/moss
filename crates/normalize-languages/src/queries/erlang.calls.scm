; Erlang calls query
; @call — call expression nodes
; @call.qualifier — qualifier/receiver for remote calls
;
; Erlang has two call forms:
;   - Local call: func(Args)  — represented as `call` with atom target
;   - Remote call: module:func(Args) — represented as `remote` with module and function

; Local call: func(Args)
; The callee field on `call` is `expr:`, not `target:`.
(call
  expr: (atom) @call)

; Local call through a variable-bound function value: Fun(Args) (the
; common higher-order-function/callback idiom — passing a `fun` around and
; invoking it later). Verified via `normalize syntax query`: `call.expr`
; allows `var` in addition to `atom` per node-types.json's `_expr`
; supertype list.
(call
  expr: (var) @call)

; Remote call: module:func(Args)
; `module:` wraps the module atom in a `remote_module` node (whose full text
; includes the trailing `:`), so we descend into it to capture just the atom.
; The function field is `fun:`, not `function:`.
(remote
  module: (remote_module (atom) @call.qualifier)
  fun: (atom) @call)

; Dynamic remote call with a variable module qualifier: Mod:func(Args).
; `remote_module.module` and `remote.fun` are both the `_expr_max`
; supertype (verified via `normalize syntax ast`), which allows `var` in
; addition to `atom` — dynamic dispatch (behaviour/plugin-style code
; calling `Mod:Fun(Args)` or `Mod:foo(Args)`) is an idiomatic, common
; pattern, not an edge case.
(remote
  module: (remote_module (var) @call.qualifier)
  fun: (atom) @call)

(remote
  module: (remote_module (atom) @call.qualifier)
  fun: (var) @call)

(remote
  module: (remote_module (var) @call.qualifier)
  fun: (var) @call)
