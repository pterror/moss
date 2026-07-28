; Go imports query
; @import       — the entire import spec (for line number)
; @import.path  — the quoted module path (quotes stripped by Rust)
; @import.alias — package alias (package_identifier)
; @import.glob  — dot import marker (presence means is_wildcard=true)
;
; import_spec.path is grammar-documented as allowing both
; interpreted_string_literal (`"pkg"`) and raw_string_literal (`` `pkg` ``).
; Raw-string import paths are vanishingly rare in real Go (gofmt/goimports
; never produce them), but they are grammar-legal and cheap to cover, so
; all four forms below accept either string kind.

; Single import: import "pkg"
(import_spec
  path: [(interpreted_string_literal) (raw_string_literal)] @import.path) @import

; Aliased import: import alias "pkg"
(import_spec
  name: (package_identifier) @import.alias
  path: [(interpreted_string_literal) (raw_string_literal)] @import.path) @import

; Dot import: import . "pkg"
(import_spec
  name: (dot) @import.glob
  path: [(interpreted_string_literal) (raw_string_literal)] @import.path) @import

; Blank import: import _ "pkg" (alias is _)
(import_spec
  name: (blank_identifier) @import.alias
  path: [(interpreted_string_literal) (raw_string_literal)] @import.path) @import
