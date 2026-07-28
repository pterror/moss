; Kotlin type references
; Captures type identifiers used in type positions.

; Plain type identifiers: Foo, String. This single blanket pattern already
; matches every `type_identifier` node in the tree regardless of nesting
; (tree-sitter query patterns with no parent constraint match anywhere),
; including ones wrapped in `user_type` (qualified/generic types like
; `LinkedList<T>`, `foo.Bar`). A previously-separate
; `(user_type (type_identifier) @type.reference)` pattern was therefore a
; strict subset of this one and produced a literal duplicate @type.reference
; capture for every qualified/generic type usage — the common case in real
; Kotlin code (any generic container, any dotted type). Removed rather than
; kept "for clarity": a redundant pattern that doubles every match it covers
; is a bug, not documentation.
(type_identifier) @type.reference

; Type-defining declarations: class, object, and type alias are all
; definitions of a named type (mirrors the java/go/rust convention). Note
; the declared name is also a `type_identifier` and therefore additionally
; matches the blanket @type.reference pattern above — expected, not a
; duplicate-match bug: same precedent as rust.types.scm's struct/enum
; names, which are both @name/@definition.type AND separately picked up by
; its own blanket `(type_identifier) @type.reference` pattern. Function
; declarations are intentionally excluded — a function name is not itself a
; type.
(class_declaration
  (type_identifier) @name) @definition.type

(object_declaration
  (type_identifier) @name) @definition.type

(type_alias
  (type_identifier) @name) @definition.type
