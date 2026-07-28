; Elixir calls query
; @call — call expression nodes
; @call.qualifier — qualifier/receiver for remote calls
;
; In Elixir's tree-sitter grammar, all function calls are represented as
; `call` nodes. The target can be an identifier (local call) or a dot node
; (remote call: Module.func() or var.func()). This includes macro calls
; like def/defmodule/if/case — they are all calls in Elixir.

; Local call: foo() or foo(args)
(call
  target: (identifier) @call)

; Remote call: Module.func() or obj.func()
(call
  target: (dot
    left: (_) @call.qualifier
    right: (identifier) @call))

; Anonymous-function invocation: `add_one.(5)` — `fun.()` syntax for calling
; a function value bound to a variable. `dot.right` is optional in this
; grammar and is entirely absent for this form (verified via `normalize
; syntax query`: `add_one.(5)` parses as `call target: (dot left:
; (identifier), no right)`, distinct from the remote-call `dot` shape above
; which always has `right: (identifier)`). `!right` asserts the field is
; absent so this pattern never re-fires on (and double-counts) the
; remote-call pattern above. Only the simple `identifier`-bound case is
; captured as a clean call name; `get_adder().(3)` (target: dot left: (call))
; has no simple name to extract and is intentionally left unmatched rather
; than fabricating one from arbitrary call-expression text.
(call
  target: (dot
    left: (identifier) @call
    !right))

; Dynamic/macro-generated call target: `unquote(name)(1, 2)` — grammar-legal
; (`call.target` allows `call` in addition to `identifier`/`dot`, verified
; via `normalize syntax query` on macro-generated code such as
; `quote do: unquote(name)(1, 2) end`). The callee name is not statically
; known; captured as a best-effort partial reference using the inner call's
; own source text (e.g. "unquote(name)") rather than silently dropped,
; mirroring the precedent set for Ruby's dynamic-superclass capture.
(call
  target: (call) @call)
