; GLSL tags query
;
; GLSL is C-like: functions use function_declarator, structs use struct_specifier.

(function_definition
  declarator: (function_declarator
    declarator: (identifier) @name)) @definition.function

(struct_specifier
  name: (type_identifier) @name
  body: (_)) @definition.class

; Interface blocks: `uniform Name { ... } instance;`, `buffer Name { ... } ssbo;`,
; `in Name { ... } vs_in;`, `out Name { ... } vs_out;`. Extremely common in
; real GLSL (UBOs, SSBOs, shader-stage I/O blocks) but the grammar gives them
; no `struct_specifier` node — they parse as a plain `declaration` whose
; storage-qualifier keyword, block-name identifier, member list, and instance
; name are all *unfielded* children (verified via `node-types.json`: the
; `declaration` node has no `name`/`type` field covering this shape). Matched
; positionally instead: qualifier keyword, then an identifier, then the
; member list — this is unambiguous because a plain (non-block) qualified
; declaration never has a `field_declaration_list` child.
(declaration
  "uniform"
  (identifier) @name
  (field_declaration_list)) @definition.class

(declaration
  "buffer"
  (identifier) @name
  (field_declaration_list)) @definition.class

(declaration
  "in"
  (identifier) @name
  (field_declaration_list)) @definition.class

(declaration
  "out"
  (identifier) @name
  (field_declaration_list)) @definition.class
