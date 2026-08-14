; AsciiDoc imports query
; @import       — the entire include:: macro (for line number)
; @import.path  — the path argument of the include macro
;
; The file path lives in the `target` node (always present, required,
; multiple: 1). `block_macro_attr` holds the bracketed attribute list
; (e.g. `[lines=1..10]`, `[tag=foo]`) and is optional/absent for the common
; bare form `include::path[]` — matching on it (as a previous version of
; this query did) silently dropped every attribute-less include and, when
; attributes were present, captured the attribute string instead of the
; path.

; include::path/to/file.adoc[]
; include::path/to/file.adoc[lines=1..10]
(block_macro
  (block_macro_name) @_name
  (#eq? @_name "include")
  (target) @import.path) @import
