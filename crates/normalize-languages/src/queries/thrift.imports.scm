; Thrift imports query
; @import       — the entire include statement (for line number)
; @import.path  — the included file path

; include "shared.thrift"
; cpp_include "custom.h"     -- verified via `normalize syntax ast`: the
;   `cpp_include` keyword produces the exact same `include_statement` node
;   type as plain `include` (only the leading literal token differs), so
;   this single pattern already covers both without needing a separate
;   clause.
; The quoted path is a `string` node (quotes stripped by Rust); `literal`
; is not a node type this grammar produces here.
;
; `package_declaration` (`package "com.example.pkg"`) is a distinct header
; node -- it names the compilation unit's package for codegen, it does not
; reference another file, so it is not an import and is intentionally not
; matched here (mirrors namespace_declaration also being out of scope).
(include_statement
  (string) @import.path) @import
