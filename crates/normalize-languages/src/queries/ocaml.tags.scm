; OCaml tags query
; Covers: value/function definitions, type definitions, module definitions,
; module type (signature) definitions, exception definitions, class and
; class-type definitions, object methods/instance variables, and FFI
; (`external`) declarations.
;
; Field-completeness notes (verified via `normalize syntax ast`/
; `normalize syntax query` against arborium-ocaml 2.17.0's node-types.json):
;
; - `let_binding.pattern` is typed `_binding_pattern`, a 9-variant
;   supertype. Only `value_name` (plain `let f = ...`) yields a single
;   static name worth tagging as a definition; `tuple_pattern`,
;   `cons_pattern`, `constructor_pattern`, `or_pattern`, `list_pattern`
;   (destructuring binds) bind zero or multiple names and are intentionally
;   NOT tagged here (no single name to report), same as other languages'
;   tags queries skip destructuring lets. `parenthesized_operator`
;   (operator definitions, e.g. `let ( +. ) a b = ...` — a common pattern
;   for defining infix operators, e.g. monadic `let ( >>= ) m f = ...`) and
;   `typed_pattern` (a parenthesized type-annotated name, e.g.
;   `let (x : int) = 5`, wrapping either a `value_name` or a
;   `parenthesized_operator`) DO carry a single static name and were
;   previously dropped entirely.
; - `type_binding.name` is typed `type_constructor` OR
;   `type_constructor_path` — the latter is what a type *extension*
;   (`type t += Foo of int`, used for extensible variant types) parses as;
;   previously unmatched.
; - `external` (FFI/primitive declarations) was entirely unhandled despite
;   binding a name into scope exactly like `value_definition`; its `name`
;   position is an unnamed child, not a field, and (like `let_binding`) can
;   be `value_name` or `parenthesized_operator` (stdlib-style primitive
;   operator externals, e.g. `external ( + ) : int -> int -> int =
;   "%addint"`).
; - `exception_definition`, `class_definition`, `class_type_definition`,
;   `method_definition`/`method_specification`,
;   `instance_variable_definition`, and `value_specification` (the `val`
;   declarations inside a `module type`/`.mli` signature) were entirely
;   unhandled — OCaml's exception, class/object system, and `.mli`
;   interface files have no coverage without them.

; Value and function definitions (OCaml doesn't syntactically distinguish)
(value_definition
  (let_binding
    pattern: (value_name) @name)) @definition.function

; Operator definitions: let ( +. ) a b = ...
(value_definition
  (let_binding
    pattern: (parenthesized_operator) @name)) @definition.function

; Type-annotated let: let (x : int) = 5 / let ((+.) : t) = ...
(value_definition
  (let_binding
    pattern: (typed_pattern
      pattern: [(value_name) (parenthesized_operator)] @name))) @definition.function

; FFI / primitive declarations: external raw : int -> int = "raw_impl"
(external
  [(value_name) (parenthesized_operator)] @name) @definition.function

; Signature value declarations (module type / .mli): val f : int -> int
(value_specification
  (value_name) @name) @definition.function

; Type definitions (includes records, variants, aliases)
(type_definition
  (type_binding
    name: (type_constructor) @name)) @definition.type

; Type extensions: type t += Foo of int
(type_definition
  (type_binding
    name: (type_constructor_path
      (type_constructor) @name))) @definition.type

; Module definitions
(module_definition
  (module_binding
    (module_name) @name)) @definition.module

; Module type definitions (signatures = interfaces)
(module_type_definition
  (module_type_name) @name) @definition.interface

; Exception definitions — no dedicated @definition.exception vocabulary
; exists across the language pack; follows the precedent already set by
; thrift.tags.scm mapping exception_definition to @definition.class.
(exception_definition
  (constructor_declaration
    (constructor_name) @name)) @definition.class

; Class definitions: class counter = object ... end
(class_definition
  (class_binding
    (class_name) @name)) @definition.class

; Class type definitions: class type counter_type = object ... end
(class_type_definition
  (class_type_binding
    (class_type_name) @name)) @definition.interface

; Object/class methods
(method_definition
  (method_name) @name) @definition.method

; Class-type method specifications: method get : int
(method_specification
  (method_name) @name) @definition.method

; Object/class instance variables
(instance_variable_definition
  (instance_variable_name) @name) @definition.var
