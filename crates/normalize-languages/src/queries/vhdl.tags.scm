; VHDL tags query
;
; Name-field variants (verified against node-types.json + real parse output,
; see crates/normalize-languages/tests/query_fixtures.rs
; vhdl_tags_completeness): `entity_declaration.name`,
; `architecture_body.name`, `package_declaration.name`, and
; `full_type_declaration.name` all allow `extended_identifier`
; (VHDL's `\Foo Bar\` escaped-identifier syntax) in addition to plain
; `identifier`. `function_body.designator` / `procedure_body.designator`
; additionally allow `operator_symbol` (operator overloading, e.g.
; `function "+"`) on top of `identifier` / `extended_identifier`.

; Entity declarations
(entity_declaration
  name: [(identifier) (extended_identifier)] @name) @definition.class

; Architecture bodies
(architecture_body
  name: [(identifier) (extended_identifier)] @name) @definition.class

; Package declarations
(package_declaration
  name: [(identifier) (extended_identifier)] @name) @definition.module

; Full type declarations
(full_type_declaration
  name: [(identifier) (extended_identifier)] @name) @definition.type

; Function bodies (use designator field)
(function_body
  designator: [(identifier) (extended_identifier) (operator_symbol)] @name) @definition.function

; Procedure bodies (use designator field)
(procedure_body
  designator: [(identifier) (extended_identifier) (operator_symbol)] @name) @definition.function
