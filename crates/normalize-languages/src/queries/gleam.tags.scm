; Gleam tags query
; Covers: functions, type definitions, type aliases, constants

; Function definitions
(function
  name: (identifier) @name) @definition.function

; Type definitions (ADTs / custom types)
(type_definition
  (type_name name: (type_identifier) @name)) @definition.class

; Type aliases
(type_alias
  (type_name name: (type_identifier) @name)) @definition.type

; Constants
(constant
  name: (identifier) @name) @definition.constant
