; Java calls query
; @call — call expression nodes
; @call.qualifier — qualifier/receiver for method calls

; Method invocation: obj.method() or method()
(method_invocation
  object: (_) @call.qualifier
  name: (identifier) @call)

(method_invocation
  name: (identifier) @call)

; Explicit constructor invocation: super(...) / this(...) constructor
; delegation. This is a distinct node kind (`explicit_constructor_invocation`,
; not `method_invocation`) so it was entirely unmatched — a real, common gap
; since every subclass constructor calling `super(...)` (or delegating via
; `this(...)`) silently disappeared from call extraction.
(explicit_constructor_invocation
  constructor: (super) @call)

(explicit_constructor_invocation
  constructor: (this) @call)
