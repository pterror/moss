; GraphQL tags — definitions for types, interfaces, enums, unions, inputs, scalars, operations
; GraphQL grammar nodes have no fields — the name is a positional (name) child.

(object_type_definition
  (name) @name) @definition.class

(interface_type_definition
  (name) @name) @definition.interface

(enum_type_definition
  (name) @name) @definition.type

(union_type_definition
  (name) @name) @definition.type

(input_object_type_definition
  (name) @name) @definition.class

(scalar_type_definition
  (name) @name) @definition.type

(operation_definition
  (name) @name) @definition.function

(fragment_definition
  (fragment_name (name) @name)) @definition.function

(field_definition
  (name) @name) @definition.method

; Directive definitions: `directive @auth(role: String) on FIELD_DEFINITION`.
; Declares a new attachable annotation invoked elsewhere as `@name(...)` —
; the closest existing kind by behavior (a reusable, argument-taking construct
; applied to other declarations) is @definition.macro, mirroring how
; attribute/decorator-defining constructs are mapped in other grammars
; (java.tags.scm maps the analogous `@interface` annotation-type declaration
; to the closest *type-shaped* existing kind; here the shape is closer to an
; invocable macro than to a type, since directives take arguments and are
; invoked with `@name(...)`, not instantiated or subtyped).
(directive_definition
  (name) @name) @definition.macro

; Type extensions (`extend type Foo { ... }`, `extend interface`, etc.) are
; distinct node types from their base `_definition` counterparts in this
; grammar (confirmed via node-types.json: object_type_extension is a sibling
; of object_type_definition, not a variant of it) and were previously
; entirely unmatched. Tagged with the same @definition.* kind as the
; construct they extend so an outline groups a type's definition and its
; extensions together.
(object_type_extension
  (name) @name) @definition.class

(interface_type_extension
  (name) @name) @definition.interface

(enum_type_extension
  (name) @name) @definition.type

(union_type_extension
  (name) @name) @definition.type

(input_object_type_extension
  (name) @name) @definition.class

(scalar_type_extension
  (name) @name) @definition.type

; schema_definition / schema_extension (`schema { query: Query ... }`) are
; intentionally NOT tagged here: per node-types.json, schema_definition has
; no `name` child at all (verified — its only children are description,
; directives, root_operation_type_definition) since a document has at most
; one schema block. There is no name to attach a @definition.* tag to; this
; is a genuine grammar-level absence, not an oversight.
