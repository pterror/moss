; Ruby calls query
; @call — call expression nodes
; @call.qualifier — qualifier/receiver for method calls

; Simple method call (no receiver): func or func(args)
(call
  method: (identifier) @call)

; Bare Kernel-style calls where the callee name is itself a constant, not an
; identifier: Integer(x), Array(x), String(x), Float(x). `call`'s `method`
; field allows `_variable` (identifier | constant | self | super | ivar/cvar/
; gvar) in addition to `operator` — verified via `normalize syntax query`
; that these parse as ordinary `call` nodes with method: (constant).
(call
  method: (constant) @call)

; Method call with receiver: obj.method or obj.method(args). This pattern
; captures @call.qualifier only — @call itself is deliberately NOT
; recaptured here. The plain `(call method: (identifier) @call)` pattern
; above has no constraint on the `receiver` field at all, so it already
; matches every call regardless of whether a receiver is present; adding a
; second @call capture on the receiver-qualified pattern silently
; double-counted every receiver-qualified call (verified via
; `collect_captures_full` producing two identical @call entries for the
; same node — a real, previously-silent extraction-depth bug, not a
; hypothetical one).
(call
  receiver: (_) @call.qualifier
  method: (identifier))

; Explicit operator-method calls (`a.+(b)`, `a.<=>(b)`) use `method:
; (operator)` and are grammar-legal, but idiomatic Ruby almost always writes
; these as the `binary` form (`a + b`, `a <=> b`) instead, which is a
; different node type entirely (not a `call`) and out of scope for a calls
; query. The explicit-receiver form is rare enough in real code that it's
; not worth a dedicated pattern; documented here rather than silently
; guessed at.
