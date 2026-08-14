; Erlang imports query
; @import       — the entire attribute (for line number)
; @import.path  — the module being imported

; -import(module, [fun/arity, ...]).
(import_attribute
  module: (atom) @import.path) @import

; -import(?MOD_MACRO, [fun/arity, ...]). — `import_attribute.module` is the
; `_name` supertype (atom | macro_call_expr | var); a macro-named import
; module parses cleanly (verified via `normalize syntax query`), captured
; as the macro invocation text (best static representation available).
(import_attribute
  module: (macro_call_expr) @import.path) @import
