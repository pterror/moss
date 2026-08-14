; Type reference query for F#
; Captures type identifiers used in type annotations and definitions.

; Simple types: int, string, MyType
(simple_type
  (long_identifier) @type.reference)

; Generic types: List<int>, Option<string>
(generic_type
  (long_identifier) @type.reference)

; Type-test patterns: `:? System.DivideByZeroException` (in a `with` clause
; of a try/match). The grammar's `_type` supertype has 12 variants
; (node-types.json: anon_record_type, atomic_type, compound_type,
; constrained_type, flexible_type, function_type, generic_type, list_type,
; paren_type, postfix_type, simple_type, static_type); most of them wrap a
; nested `simple_type`/`generic_type` and so are already covered by the two
; patterns above via tree-sitter's any-depth matching, but `atomic_type`
; (used specifically by `:?` type-test patterns) holds `long_identifier`
; directly as its own child rather than wrapping a `simple_type` — verified
; via `normalize syntax ast`/`normalize syntax query`: the exact `:?
; System.DivideByZeroException` clause already present in this crate's own
; `sample.fs` fixture produced zero @type.reference captures before this
; addition.
(atomic_type
  (long_identifier) @type.reference)
