; PHP type reference query
; Captures type names used in type annotations (PHP 7+ typed properties,
; parameter types, return types, and union types).

; Named type: Foo, Bar, int (in type position)
(named_type
  (name) @type.reference)

; Qualified name as type: \Foo\Bar
(named_type
  (qualified_name) @type.reference)

; Namespace-relative type: namespace\LocalType. `named_type`'s children
; allow name/qualified_name/relative_name; relative_name was the only
; variant not handled — confirmed unmatched via `normalize syntax query`.
(named_type
  (relative_name) @type.reference)

; Primitive type: int, string, bool, float, etc.
(primitive_type) @type.reference

; Union (Foo|Bar), intersection (Foo&Bar), optional (?Foo), and DNF
; ((Foo&Bar)|Baz) types are all wrapper nodes whose leaves are named_type/
; primitive_type — the two unanchored rules above already reach into them
; regardless of which wrapper node they're nested under (verified via
; `normalize syntax query` for all four forms). A previous explicit
; `(union_type (named_type (name) @type.reference))` rule duplicated this:
; every union member was captured twice (once by that rule, once by the
; already-unanchored named_type rule above) — confirmed via `normalize
; syntax query` that `Foo|Bar` produced two @type.reference captures per
; name before this fix. Removed rather than kept as a second source of the
; same match.
