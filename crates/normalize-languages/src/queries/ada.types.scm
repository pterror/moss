; Ada type reference query
; Captures type marks (names used in type positions) throughout Ada code.
;
; In Ada, `subtype_mark` fields carry the type name — they appear in
; variable declarations, parameter specs, return types, type derivations, etc.

; Type name in any subtype_mark position
(object_declaration
  subtype_mark: (identifier) @type.reference)

(parameter_specification
  subtype_mark: (identifier) @type.reference)

(result_profile
  subtype_mark: (identifier) @type.reference)

(component_definition
  subtype_mark: (identifier) @type.reference)

(subtype_declaration
  subtype_mark: (identifier) @type.reference)

(derived_type_definition
  subtype_mark: (identifier) @type.reference)

; Package-qualified type: Package.TypeName
; Every `subtype_mark`-bearing field allows `selected_component` per
; node-types.json, not just object_declaration/parameter_specification —
; result_profile (return types), component_definition (record fields),
; subtype_declaration, and derived_type_definition were previously missing
; this variant, silently dropping every qualified return type / record
; field type / subtype / derived-type base that uses a package-qualified
; name (e.g. `function Get return Ada.Text_IO.File_Type;`).
(object_declaration
  subtype_mark: (selected_component
    selector_name: (identifier) @type.reference))

(parameter_specification
  subtype_mark: (selected_component
    selector_name: (identifier) @type.reference))

(result_profile
  subtype_mark: (selected_component
    selector_name: (identifier) @type.reference))

(component_definition
  subtype_mark: (selected_component
    selector_name: (identifier) @type.reference))

(subtype_declaration
  subtype_mark: (selected_component
    selector_name: (identifier) @type.reference))

(derived_type_definition
  subtype_mark: (selected_component
    selector_name: (identifier) @type.reference))
