; Type reference query for Scala
; Captures type identifiers used in type positions.

; Plain type identifiers: Foo, String. This single unconstrained clause also
; matches every `type_identifier` nested inside a `stable_type_identifier`
; (qualified type, e.g. `foo.Bar`) — tree-sitter queries match nodes anywhere
; in the tree regardless of their parent's kind. A previous version of this
; file had a second clause specifically for `(stable_type_identifier
; (type_identifier))`, which double-captured every qualified type reference
; (verified via `normalize syntax query`: both clauses independently matched
; the same "Date" node in `java.util.Date`). Do not re-add that clause.
(type_identifier) @type.reference
