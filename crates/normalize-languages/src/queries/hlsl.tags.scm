; HLSL tags query
;
; HLSL is C-like: functions use function_declarator, structs and cbuffers are type containers.

(function_definition
  declarator: (function_declarator
    declarator: (identifier) @name)) @definition.function

(struct_specifier
  name: (type_identifier) @name
  body: (_)) @definition.class

(cbuffer_specifier
  name: (type_identifier) @name
  body: (_)) @definition.class

; `class Foo { ... }` (Shader Model 4/5 classes, used with `interface` for
; dynamic shader linkage). The cpp-derived grammar models this with a real
; class_specifier node.
(class_specifier
  name: (type_identifier) @name
  body: (_)) @definition.class

; Methods defined inside a class body use `field_identifier` for the
; declarator name (not `identifier`, which is only used by free functions) —
; verified via node-types.json's `function_declarator.declarator` field,
; which allows both, and confirmed against real parse output of a `class`
; body member function.
(function_definition
  declarator: (function_declarator
    declarator: (field_identifier) @name)) @definition.function

; `interface Foo { ... }` has no distinct grammar node: this cpp-derived
; grammar doesn't recognize the `interface` keyword, so it mis-parses as a
; function_definition with return type `interface` (as a bare type_identifier)
; and `Foo` as a parameterless identifier declarator — indistinguishable from
; a malformed function definition. Left uncaptured; this is a grammar
; limitation, not a query gap.

; Top-level `cbuffer Name { ... };` / `tbuffer Name { ... };` with NO
; trailing `: register(...)` binding. Neither keyword is wired into this
; grammar's top-level declaration production — `cbuffer_specifier` (above)
; is only reachable as a *statement* subtype (nested inside a function
; body); at translation-unit scope both keywords mis-parse exactly like
; `interface` does: a function_definition whose "return type" is the bare
; keyword and whose declarator is a parameterless identifier. Filtered by
; #eq? so it can't collide with that mis-parse or a genuine function.
(function_definition
  type: (type_identifier) @_kw
  declarator: (identifier) @name
  body: (compound_statement)
  (#eq? @_kw "cbuffer")) @definition.class

(function_definition
  type: (type_identifier) @_kw
  declarator: (identifier) @name
  body: (compound_statement)
  (#eq? @_kw "tbuffer")) @definition.class

; KNOWN GAP, not fixable at the query layer: when a top-level `cbuffer`/
; `tbuffer` carries a `: register(...)` binding — the overwhelmingly common
; real-world shape — the grammar's GLR resolution disconnects the `{ ... }`
; body entirely: it parses as a *sibling* `compound_statement` of the
; enclosing `declaration` node, with no field or anchor relationship tying
; it back to the block's name. No tree-sitter query pattern can recover
; that association without matching on `translation_unit` child order,
; which breaks the moment anything else sits between the two nodes.
; Verified via `normalize syntax ast` on `cbuffer Foo : register(b0) { ... };`.
