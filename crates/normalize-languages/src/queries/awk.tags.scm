; AWK tags query

(func_def
  name: (identifier) @name) @definition.function

; Namespace-qualified function definition (gawk extension):
; `function mylib::add(a, b) { ... }`. `func_def.name` allows
; `ns_qualified_name` in addition to `identifier` per node-types.json —
; verified via `normalize syntax ast` that a namespaced function
; definition parses as `func_def name: (ns_qualified_name ...)`, entirely
; unmatched by the plain-identifier pattern above. awk.calls.scm already
; handled the matching namespace-qualified *call* form
; (`func_call name: (ns_qualified_name)`); the definition side was missing.
(func_def
  name: (ns_qualified_name) @name) @definition.function
