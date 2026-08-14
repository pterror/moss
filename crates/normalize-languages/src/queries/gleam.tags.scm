; Gleam tags query
; Covers: functions, type definitions, type aliases, constants

; Function definitions
(function
  name: (identifier) @name) @definition.function

; External function definitions (legacy `external fn name(...) -> T = "erl" "fn"`
; FFI-binding syntax). Distinct node type from `function` in node-types.json;
; still a live grammar production (parses without error), so it's captured
; the same as any other function definition.
(external_function
  name: (identifier) @name) @definition.function

; Type definitions (ADTs / custom types)
;
; type_name's `name` field allows both `type_identifier` (the normal case,
; `pub type Body { ... }`) and `remote_type_identifier` (`type foo.Bar { ... }`
; — not valid Gleam semantically, but the grammar parses it without error;
; verified via `normalize syntax ast` / `normalize syntax query`). Handling
; both keeps tag extraction from silently dropping the symbol on malformed
; input instead of erroring.
(type_definition
  (type_name name: [(type_identifier) (remote_type_identifier)] @name)) @definition.class

; Type aliases
(type_alias
  (type_name name: [(type_identifier) (remote_type_identifier)] @name)) @definition.type

; Constants
(constant
  name: (identifier) @name) @definition.constant
