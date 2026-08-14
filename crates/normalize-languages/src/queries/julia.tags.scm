; Julia tags query
; Julia grammar has few named fields. module_definition has "name:" field,
; but most others use positional children. signature node has no named children.

; Module definitions
(module_definition
  name: (identifier) @name) @definition.module

; Function definitions: function foo(...) ... end
; signature has no named children — captured as @name, node_name() extracts the name
(function_definition
  (signature) @name) @definition.function

; Short function definitions: foo(x) = x + 1
(assignment
  . (call_expression
    . (identifier) @name)) @definition.function

; Short function definitions with a return-type annotation:
; foo(x)::Int = x + 1 — the assignment's LHS is a typed_expression wrapping
; the call_expression, not the call_expression directly. Verified via
; `normalize syntax ast`/`syntax query` against a probe file: without this
; pattern, every type-annotated short-form definition (a common Julia idiom
; for library code with typed return values) went entirely untagged.
(assignment
  . (typed_expression
    . (call_expression
      . (identifier) @name))) @definition.function

; Macro definitions: macro foo(...) ... end
; Same structure as function_definition
(macro_definition
  (signature) @name) @definition.macro

; Struct definitions: struct Foo ... end
; Name is inside type_head
(struct_definition
  (type_head) @name) @definition.class

; Abstract type definitions: abstract type Foo end
(abstract_definition
  (type_head) @name) @definition.interface

; Module-level constants: const MAX_SIZE = 100 / const PI::Float64 = 3.14
; const_statement wraps an assignment; node_name() (julia.rs) unwraps the
; assignment's LHS (through typed_expression when a return-type-style
; annotation is present) to get the bare identifier.
(const_statement
  (assignment
    . (identifier) @name)) @definition.constant

(const_statement
  (assignment
    . (typed_expression
      . (identifier) @name))) @definition.constant
