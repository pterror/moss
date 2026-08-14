; Ada tags query
;
; node-types.json (arborium-ada 2.17.0): the `name` field on
; package_declaration/package_body/function_specification/
; procedure_specification allows 13 node-type variants (attribute_designator,
; character_literal, function_call, gnatprep_identifier, identifier,
; qualified_expression, reduction_attribute_designator, selected_component,
; slice, string_literal, target_name, tick, value_sequence). Only two are
; realistic for a declared name: `identifier` (plain name) and
; `selected_component` (dotted child-unit name, e.g. `package Parent.Child
; is` / `procedure Parent.Child is`). The rest describe attribute references
; and literals that node-types.json permits structurally but that never
; arise as a declaration name in real Ada source.

(package_declaration
  name: (identifier) @name) @definition.module

(package_declaration
  name: (selected_component) @name) @definition.module

(package_body
  name: (identifier) @name) @definition.module

(package_body
  name: (selected_component) @name) @definition.module

; NOTE: no dedicated pattern for `(generic_package_declaration
; (package_declaration ...))` — the plain package_declaration pattern above
; already matches the nested package_declaration node regardless of whether
; its parent is a generic_package_declaration (tree-sitter queries match by
; node type, not by parent). A previously-existing generic-specific pattern
; here duplicated every generic package's @definition.module capture.

(subprogram_declaration
  (function_specification
    name: (identifier) @name)) @definition.function

(subprogram_declaration
  (function_specification
    name: (selected_component) @name)) @definition.function

(subprogram_declaration
  (procedure_specification
    name: (identifier) @name)) @definition.function

(subprogram_declaration
  (procedure_specification
    name: (selected_component) @name)) @definition.function

(subprogram_body
  (function_specification
    name: (identifier) @name)) @definition.function

(subprogram_body
  (function_specification
    name: (selected_component) @name)) @definition.function

(subprogram_body
  (procedure_specification
    name: (identifier) @name)) @definition.function

(subprogram_body
  (procedure_specification
    name: (selected_component) @name)) @definition.function

(expression_function_declaration
  (function_specification name: (identifier) @name)) @definition.function

(expression_function_declaration
  (function_specification name: (selected_component) @name)) @definition.function

; Generic subprograms: `generic ... procedure Foo (...);` / `generic ...
; function Foo (...) return T;`. Previously entirely unmatched — every
; generic subprogram declaration in a codebase was silently dropped from
; tags.
(generic_subprogram_declaration
  (function_specification
    name: (identifier) @name)) @definition.function

(generic_subprogram_declaration
  (function_specification
    name: (selected_component) @name)) @definition.function

(generic_subprogram_declaration
  (procedure_specification
    name: (identifier) @name)) @definition.function

(generic_subprogram_declaration
  (procedure_specification
    name: (selected_component) @name)) @definition.function

(full_type_declaration
  (identifier) @name) @definition.type

(private_type_declaration
  (identifier) @name) @definition.type

(incomplete_type_declaration
  (identifier) @name) @definition.type
