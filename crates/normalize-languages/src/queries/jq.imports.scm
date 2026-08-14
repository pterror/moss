; jq imports query
; @import       — the entire import statement (for line number)
; @import.path  — the module path string
; @import.alias — the "as name"/"as $name" alias

; import "lib/utils" as utils;  (bare identifier alias — used to reference
; the module's functions unqualified within its namespace)
(import_
  (string) @import.path
  (identifier) @import.alias) @import

; import "lib/utils" as $utils;  (variable alias — imports the module's
; top-level `constant`/data definitions as a bound value instead)
(import_
  (string) @import.path
  (variable) @import.alias) @import

; import "lib/utils";  (no alias at all — legal per the grammar, though jq
; itself requires `as` for `import`; `include "lib/utils";` uses the same
; `import_` node with no alias, since `include` never takes one)
(import_
  (string) @import.path) @import
