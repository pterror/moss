; Thrift imports query
; @import       — the entire include statement (for line number)
; @import.path  — the included file path

; include "shared.thrift"
; The quoted path is a `string` node (quotes stripped by Rust); `literal`
; is not a node type this grammar produces here.
(include_statement
  (string) @import.path) @import
