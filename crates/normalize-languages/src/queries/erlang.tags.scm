; Erlang tags query
; Covers: function clauses, module declarations, record declarations, type aliases
;
; The `name:`/`module:` fields below are all the `_name` grammar supertype
; (verified via node-types.json + `normalize syntax query`), whose variants
; are `atom | macro_call_expr | var`. `var` never realistically appears in
; these positions in valid Erlang (module/function/record/type identity
; must be a compile-time atom, not a runtime variable — the grammar only
; allows `var` here because `_name` is shared with genuinely
; variable-nameable positions elsewhere, e.g. self-referencing named funs).
; `macro_call_expr` DOES appear in real code — macro-generated names (e.g.
; `-record(?REC_NAME, {...}).`, `?NAME(X) -> X.`) parse cleanly with no
; ERROR node — so it is handled below; the captured text is the macro
; invocation itself (`?REC_NAME`), the best static representation available
; without macro-expanding.

; Function clauses
(function_clause
  name: (atom) @name) @definition.function

(function_clause
  name: (macro_call_expr) @name) @definition.function

; Module declaration: -module(name).
(module_attribute
  name: (atom) @name) @definition.module

(module_attribute
  name: (macro_call_expr) @name) @definition.module

; Record declarations: -record(name, {...}).
(record_decl
  name: (atom) @name) @definition.class

(record_decl
  name: (macro_call_expr) @name) @definition.class

; Type aliases: -type name() :: ...
(type_alias
  name: (type_name
    name: (atom) @name)) @definition.type

(type_alias
  name: (type_name
    name: (macro_call_expr) @name)) @definition.type
