; Scala tags query
; Covers: functions, methods, classes, objects, traits, enums, type definitions

; Function definitions (top-level and inside containers)
(function_definition
  name: (identifier) @name) @definition.function

; Operator-named function/method definitions: `def +(other: Point): Point = ...`,
; `def unary_- : Point = ...` (unary_- happens to lex as a plain identifier, but
; symbolic operator names like `+`/`-`/`*` are a distinct `operator_identifier`
; node — a common idiom for arithmetic/comparison overloading on case classes).
(function_definition
  name: (operator_identifier) @name) @definition.function

; Class definitions
(class_definition
  name: (identifier) @name) @definition.class

; Object definitions (singleton objects = modules)
(object_definition
  name: (identifier) @name) @definition.module

; Trait definitions (interfaces)
(trait_definition
  name: (identifier) @name) @definition.interface

; Enum definitions (Scala 3): `enum Color { case Red, Green, Blue }`. Individual
; enum cases (simple_enum_case/full_enum_case) are not tagged as separate
; definitions, matching how java.tags.scm leaves enum constants untagged.
(enum_definition
  name: (identifier) @name) @definition.enum

; Type definitions (type aliases). `type_definition.name` only ever allows
; `type_identifier` per arborium-scala's node-types.json (no operator_identifier
; variant, unlike function/class/object/trait names).
(type_definition
  name: (type_identifier) @name) @definition.type

; -----------------------------------------------------------------------------
; References
; -----------------------------------------------------------------------------

; Simple call: func()
(call_expression
  function: (identifier) @name) @reference.call

; Method call: obj.method()
(call_expression
  function: (field_expression
    field: (identifier) @name)) @reference.call

; Explicit operator-method call: obj.+(x), this.n.+(1) — `field_expression.field`
; allows `operator_identifier` in addition to `identifier`.
(call_expression
  function: (field_expression
    field: (operator_identifier) @name)) @reference.call

; Generic/type-parameterized call: func[T]()
(call_expression
  function: (generic_function
    function: (identifier) @name)) @reference.call

; Qualified generic call: obj.method[T]()
(call_expression
  function: (generic_function
    function: (field_expression
      field: (identifier) @name))) @reference.call

; Parenthesized call target: (f)(x) — `call_expression.function` allows
; `parenthesized_expression` directly (confirmed via `normalize syntax query`);
; the whole parenthesized text is captured as the reference, mirroring
; typescript.calls.scm's treatment of the same shape.
(call_expression
  function: (parenthesized_expression) @name) @reference.call

; Object instantiation: `new Foo()`, `new Stack[Int]()`, `new java.util.Date()`,
; `new java.util.HashMap[String, Int]()`. `instance_expression`'s type child is
; unfielded; these four clauses cover plain, generic, qualified, and
; generic+qualified forms confirmed via real parse output. Rarer type-position
; wrappers (`applied_constructor_type`, `compound_type`, `projected_type`,
; `singleton_type`, `structural_type`, `named_tuple_type`, `tuple_type`,
; `annotated_type`, `wildcard`) are not handled — no real-world evidence found
; for `new` targeting those shapes.
(instance_expression
  (type_identifier) @name) @reference.class

(instance_expression
  (stable_type_identifier
    (type_identifier) @name)) @reference.class

(instance_expression
  (generic_type
    (type_identifier) @name)) @reference.class

(instance_expression
  (generic_type
    (stable_type_identifier
      (type_identifier) @name))) @reference.class

; Supertype/mixin references: `class Foo extends Bar with Baz with Qux`.
; `extends_clause.type` is declared `multiple: true` in node-types.json, but in
; practice only the *first* type after `extends` actually carries the `type`
; field — every subsequent `with X` mixin is an unfielded direct child
; (confirmed via `normalize syntax query`; this is exactly the "field declared
; but not populated for every occurrence" trap the query-testing methodology
; warns about). Scala doesn't distinguish "superclass" from "mixin trait" at
; the grammar level the way Java separates extends/implements, so all
; `extends_clause` type references — fielded or not — are tagged uniformly as
; @reference.implementation. Using unconstrained (unfielded) patterns here
; covers both the fielded first type and the unfielded mixins in one clause
; without double-capturing.
(extends_clause
  (type_identifier) @name) @reference.implementation

(extends_clause
  (stable_type_identifier
    (type_identifier) @name)) @reference.implementation

(extends_clause
  (generic_type
    (type_identifier) @name)) @reference.implementation

(extends_clause
  (generic_type
    (stable_type_identifier
      (type_identifier) @name))) @reference.implementation
