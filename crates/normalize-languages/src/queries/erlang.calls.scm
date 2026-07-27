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

; Remote call: module:func(Args)
; `module:` wraps the module atom in a `remote_module` node (whose full text
; includes the trailing `:`), so we descend into it to capture just the atom.
; The function field is `fun:`, not `function:`.
(remote
  module: (remote_module (atom) @call.qualifier)
  fun: (atom) @call)
