; Lua calls query
; @call — call expression nodes
; @call.qualifier — qualifier/receiver for method calls
;
; function_call.name is grammar-typed (per arborium-lua's node-types.json) as
; one of: function_call, method_index_expression, parenthesized_expression,
; variable (a hidden supertype that expands to bracket_index_expression,
; dot_index_expression, identifier). All six concrete variants are handled
; below, verified via `normalize syntax query`/`normalize syntax ast` against
; probe files (see docs/query-testing-methodology.md).

; Simple call: func() or func(args)
(function_call
  name: (identifier) @call)

; Method call: obj:method() — colon syntax
(function_call
  name: (method_index_expression
    table: (_) @call.qualifier
    method: (identifier) @call))

; Field call: obj.func()
(function_call
  name: (dot_index_expression
    table: (_) @call.qualifier
    field: (identifier) @call))

; Computed/bracket call: handlers["foo"](), t[i]() — dispatch-table idiom,
; common in Lua for event handlers/command tables. The bracket index (field)
; is an arbitrary expression, not necessarily a static name, so — matching
; the convention in javascript.calls.scm's identical case — the whole
; bracket_index_expression is captured as @call (best-effort text) with the
; table as @call.qualifier.
(function_call
  name: (bracket_index_expression
    table: (_) @call.qualifier) @call)

; Parenthesized call target: (function() ... end)() — IIFE idiom used for
; scoping. No static name; the parenthesized expression's text is captured
; as a best-effort @call, matching javascript.calls.scm's identical case.
(function_call
  name: (parenthesized_expression) @call)

; Chained call: get_fn()() — the callee is the result of another call.
; No static name; matches javascript.calls.scm's identical case.
(function_call
  name: (function_call) @call)
