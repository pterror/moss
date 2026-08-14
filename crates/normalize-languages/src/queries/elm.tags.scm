; Elm tags query

(value_declaration
  (function_declaration_left
    (lower_case_identifier) @name)) @definition.function

; `port name : Type` declarations (JS interop boundary) — a distinct
; top-level node type (`port_annotation`), not a `value_declaration`, so
; the pattern above never matches it. Verified via `normalize syntax ast`:
; `port outgoing : String -> Cmd msg` produces zero matches from the
; `value_declaration` pattern alone. Ports are a common, idiomatic part of
; real Elm applications (the only way to talk to JS), so this is a real gap.
(port_annotation
  name: (lower_case_identifier) @name) @definition.function

(type_alias_declaration
  (upper_case_identifier) @name) @definition.type

(type_declaration
  (upper_case_identifier) @name) @definition.class
