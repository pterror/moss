; Type reference query for Ruby
; Ruby is dynamically typed; captures class names used in inheritance.

; Superclass in class definition: class Foo < Bar
(superclass
  (constant) @type.reference)

; Namespaced superclass: class Foo < Mod::Base — this is a scope_resolution,
; not a constant, but is already covered by the generic scope_resolution
; pattern below (it matches the (constant) leaves of any scope_resolution
; node regardless of parent, superclass included; verified via `normalize
; syntax query`).

; Dynamic/computed superclass: class Foo < Struct.new(:x, :y) — a call is
; grammar-legal in the superclass position (superclass's child field type is
; `_expression`, whose subtypes include `call`) and is a common idiom for
; lightweight Struct-based value classes. There is no static "name" for the
; resulting anonymous class, so as a best-effort reference we capture the
; call's own receiver constant (e.g. "Struct") rather than fabricate a name
; for the synthesized class — an honest partial signal, not a full type
; name.
(superclass
  (call
    receiver: (constant) @type.reference))

; Scope resolution: Foo::Bar — capture both parts
(scope_resolution
  (constant) @type.reference)
