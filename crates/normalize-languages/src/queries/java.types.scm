; Java type references
; Captures type identifiers used in type positions.

; Plain type identifiers: Foo, ArrayList
(type_identifier) @type.reference

; Scoped types: java.util.List — capture the leaf type_identifier
(scoped_type_identifier
  (type_identifier) @type.reference)

; Type-defining declarations: classes, interfaces, enums, records, and
; annotation types are all definitions of a named type.
(class_declaration name: (identifier) @name) @definition.type

(interface_declaration name: (identifier) @name) @definition.type

(enum_declaration name: (identifier) @name) @definition.type

(record_declaration name: (identifier) @name) @definition.type

(annotation_type_declaration name: (identifier) @name) @definition.type
