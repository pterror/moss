; Go type definitions
; Captures type names from struct and interface definitions.

; Type definition: type Stack struct {...}
(type_spec
  name: (type_identifier) @name) @definition.type

; Type alias: type MyInt = int — a distinct node type from type_spec (no
; `=` in a type_spec); type_declaration's children field allows both
; [type_alias, type_spec] but only type_spec was handled here, silently
; dropping every type-alias definition.
(type_alias
  name: (type_identifier) @name) @definition.type

; Qualified type references: io.Reader, http.Handler
(qualified_type
  package: (package_identifier) @type.qualifier
  name: (type_identifier) @name)
