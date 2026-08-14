; TSX type references (same as TypeScript)
; Captures type identifiers used in type positions.

; Plain type identifiers: Foo, Bar
(type_identifier) @type.reference

; Predefined types: string, number, boolean, void, any, never, object, symbol, bigint
(predefined_type) @type.reference

; Nested types: Foo.Bar — capture the full path
(nested_type_identifier) @type.reference

; Type definitions: class and interface
(class_declaration name: (type_identifier) @name) @definition.type
(interface_declaration name: (type_identifier) @name) @definition.interface

; Superclass/interface references: class Derived extends Base implements Logger
; (javascript.types.scm already captures `extends` via class_heritage; TS/TSX's
; class_heritage has a different internal shape — extends_clause/
; implements_clause — and was missing both entirely.)
(extends_clause
  value: (identifier) @type.reference)
(extends_clause
  value: (member_expression
    property: (property_identifier) @type.reference))
(implements_clause
  (type_identifier) @type.reference)
(implements_clause
  (generic_type
    name: (type_identifier) @type.reference))
