; Type reference query for Groovy
; Captures type identifiers used in declarations and parameters.
;
; The Groovy grammar has no dedicated `type_identifier`/`qualified_name`
; node for type positions (`qualified_name` only appears in import/package
; statements) — the `type:` field on `declaration`, `parameter`, and
; `function_definition` instead holds one of `identifier`, `builtintype`,
; `array_type`, or `type_with_generics` directly. Generic type arguments live
; one level deeper inside a `generics` node.

; Declared type of a variable/field: Foo bar = ...
(declaration type: (identifier) @type.reference)
(declaration type: (builtintype) @type.reference)
(declaration type: (array_type) @type.reference)

; Declared type of a function parameter: (Foo bar)
(parameter type: (identifier) @type.reference)
(parameter type: (builtintype) @type.reference)
(parameter type: (array_type) @type.reference)

; Return type of a function/method: Foo doThing() { ... }
(function_definition type: (identifier) @type.reference)
(function_definition type: (builtintype) @type.reference)
(function_definition type: (array_type) @type.reference)

; Generic types: List<String>, Map<String, Object>
; — the base type and each generic argument
(type_with_generics (identifier) @type.reference)
(type_with_generics (builtintype) @type.reference)
(type_with_generics (array_type) @type.reference)
(generics (identifier) @type.reference)
(generics (builtintype) @type.reference)
(generics (array_type) @type.reference)
