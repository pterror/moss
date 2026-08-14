; PowerShell imports query
; @import       — the entire import command (for line number)
; @import.path  — the module being imported

; Import-Module ModuleName
; Import-Module "path/to/module"
(command
  command_name: (command_name) @_cmd (#match? @_cmd "(?i)^import-module$")
  command_elements: (command_elements) @import.path) @import

; . ./script.ps1 (dot-sourcing)
;
; Dot-sourcing never actually produces `command_name: (command_name)` with
; text ".": the "." is its own `command_invokation_operator` child (a
; sibling of command_name, not the command_name field's content), and the
; command_name field itself is `command_name_expr` for both bare and
; quoted-path forms. Verified via `normalize syntax ast` — the previous
; version of this pattern (matching command_name: (command_name) with text
; ".") could never match anything; dot-sourcing imports went entirely
; undetected.
(command
  (command_invokation_operator) @_dot (#eq? @_dot ".")
  command_name: (command_name_expr) @import.path) @import

; using module Foo / using namespace System.Collections.Generic
;
; The arborium PowerShell grammar has no dedicated `using_statement` node
; type (verified: no node type matching /using/i in node-types.json) —
; `using module X` / `using namespace X` parse as an ordinary `command`
; node with command_name text "using" and two `generic_token` children in
; command_elements (the `module`/`namespace`/`assembly` keyword, then the
; target). Matched positionally and guarded on both the command name and
; the keyword token to avoid misfiring on a hypothetical user-defined
; `using` command/alias.
(command
  command_name: (command_name) @_using (#eq? @_using "using")
  command_elements: (command_elements
    (generic_token) @_kind (#match? @_kind "(?i)^(module|namespace|assembly)$")
    (generic_token) @import.path)) @import
