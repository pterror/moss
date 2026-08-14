; WIT (WebAssembly Interface Types) imports query
; @import       — the entire use/import/include statement (for line number)
; @import.path  — the interface/world path being referenced
; @import.name  — a specific item imported (from a braced `.{a, b}` name list)
; @import.alias — alias for a top-level `use foo:bar/baz as alias;`

; use wasi:io/streams.{input-stream, output-stream}  (inside an interface/world body)
; `use_item` always carries a `.{...}` name list in this grammar (the bareword form
; without `{...}` doesn't parse inside an interface/world body — only the top-level
; `use` statement below supports it), so `definitions` is not optional here.
(use_item
  (use_path) @import.path
  (definitions
    (use_names_item) @import.name)) @import

; top-level: use wasi:io/streams;  /  use wasi:io/streams as streams;
(toplevel_use_item
  (use_path) @import.path
  alias: (id)? @import.alias) @import

; world body: import wasi:io/streams;  (referencing an already-declared interface)
;
; NOTE: `import name: func(...)` / `import name: interface {...}` — an inline
; `extern_type` definition rather than a reference to another interface — has no
; `use_path` child and is intentionally NOT matched here: it declares a local
; signature inline, it doesn't import anything from elsewhere.
(import_item
  (use_path) @import.path) @import

; world body: include wasi:cli/base;  (merges another world's imports/exports)
; `with { name as alias, ... }` is optional, so `definitions` is wrapped in `?`.
(include_item
  (use_path) @import.path
  (definitions
    (include_names_item) @import.name)?) @import
