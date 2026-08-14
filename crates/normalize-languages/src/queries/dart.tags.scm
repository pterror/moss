; Dart tags query
; Covers: functions, methods, classes, enums, mixins, extensions

; Class definitions
(class_definition
  name: (identifier) @name) @definition.class

; Enum declarations
(enum_declaration
  name: (identifier) @name) @definition.class

; Mixin declarations (interface-like) — name is a positional identifier child, not a field
(mixin_declaration
  (identifier) @name) @definition.interface

; Extension declarations (reference)
(extension_declaration
  name: (identifier) @name) @reference.implementation

; Top-level function signatures
(function_signature
  name: (identifier) @name) @definition.function

; Method signatures inside classes are wrapped by method_signature but actual kinds are:
; function_signature, getter_signature, setter_signature inside class_body

; Getter signatures
(getter_signature
  name: (identifier) @name) @definition.method

; Setter signatures
(setter_signature
  name: (identifier) @name) @definition.method

; Constructors — `method_signature`'s children also include
; constructor_signature/factory_constructor_signature/
; constant_constructor_signature/redirecting_factory_constructor_signature,
; none of which are function_signature/getter_signature/setter_signature.
; These were entirely untagged, silently dropping every constructor
; (the single most common member kind in a real Dart class) from
; extraction. All four node kinds are unfielded for the "actual name"
; position: for an unnamed constructor (`Foo(...)`) the single identifier
; is immediately followed by formal_parameter_list; for a named
; constructor (`Foo.named(...)`) the class-qualifying identifier is
; followed by "." then the real name identifier, which is immediately
; followed by formal_parameter_list. Anchoring `@name` to "immediately
; precedes formal_parameter_list" picks the right identifier in both
; shapes with one pattern per node kind — verified via
; `normalize syntax query` against unnamed/named/factory/const forms.
(constructor_signature
  (identifier) @name .
  (formal_parameter_list)) @definition.method

(factory_constructor_signature
  (identifier) @name .
  (formal_parameter_list)) @definition.method

(constant_constructor_signature
  (identifier) @name .
  (formal_parameter_list)) @definition.method

(redirecting_factory_constructor_signature
  (identifier) @name .
  (formal_parameter_list)) @definition.method

; Operator overloads: `operator +(...)`, `operator [](...)`, `operator []=(...)`.
; The operator symbol itself is the closest thing to a "name" — binary/unary
; operators are a named `binary_operator` child; `[]`/`[]=` (index get/set)
; are anonymous literal tokens, which tree-sitter queries can still capture
; directly.
(operator_signature
  [(binary_operator) "[]" "[]="] @name) @definition.method
