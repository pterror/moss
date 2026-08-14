; Starlark (Bazel/Buck) imports query
; @import       — the entire load statement (for line number)
; @import.path  — the .bzl file path
; @import.name  — a single symbol being loaded
;
; load_statement's children are an unordered-looking but positionally fixed
; mix of `string` and `aliased_load` nodes (per arborium-starlark's
; node-types.json): the FIRST child is always the path string; every
; subsequent `string` child is a plain (non-aliased) imported symbol name,
; and every `aliased_load` child is an aliased imported symbol. The original
; query used a bare `(load_statement (string) @import.path)` with no
; anchor, which matched EVERY string child, not just the first — so
; `load("//foo:bar.bzl", "sym1", "sym2")` (the common, non-aliased form)
; misclassified "sym1" and "sym2" as import paths and never captured them
; as @import.name at all. Fixed by anchoring the path to the position right
; after "(" and adding a dedicated clause for plain (non-aliased) name
; strings, verified via `normalize syntax query`.

; load("//path/to:file.bzl", ...) — first string is always the path.
(load_statement
  "("
  .
  (string) @import.path) @import

; load("//path/to:file.bzl", "symbol", "other")
; Every string that immediately follows another string (separated by ",")
; is a plain (non-aliased) imported symbol name, not the path.
(load_statement
  (string)
  ","
  (string) @import.name) @import

; load("//path/to:file.bzl", local = "symbol")
(load_statement
  (aliased_load
    alias: (identifier) @import.name)) @import
