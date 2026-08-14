; Starlark calls query
; @call — call expression
; @call.qualifier — receiver object for method calls
;
; Starlark (Bazel build language) has Python-like syntax. Function calls are
; `call` nodes with a `function` field that is grammar-typed as the
; `primary_expression` supertype — per arborium-starlark's node-types.json
; this is NOT a hidden/inlined supertype: `normalize syntax ast` confirms a
; real `primary_expression` wrapper node is always present, with exactly one
; named child that is one of: attribute, call, dictionary,
; dictionary_comprehension, false, float, identifier, integer, list,
; list_comprehension, none, parenthesized_expression, string, subscript,
; true, tuple. Of these, four occur as realistic call targets and are
; handled below (verified via `normalize syntax query`/`normalize syntax
; ast`, matching the identical pattern already used in javascript.calls.scm
; and lua.calls.scm for the same primary-expression-as-callee shape). The
; remaining variants (dictionary, float, integer, etc.) are syntactically
; legal in the grammar but never valid call targets semantically, so they
; are not given call-extraction clauses.

; Simple call: func(args...)
(call
  function: (primary_expression
    (identifier) @call))

; Method call: obj.method(args...)
(call
  function: (primary_expression
    (attribute
      object: (_) @call.qualifier
      attribute: (identifier) @call)))

; Computed/bracket call: HANDLERS[key](args...) — dispatch-table idiom,
; common in Bazel macros for kind-based dispatch. The subscript is an
; arbitrary expression, not necessarily a static name, so the whole
; subscript node is captured as @call (best-effort text) with the base
; value as @call.qualifier — matching the convention in
; javascript.calls.scm/lua.calls.scm's identical case.
(call
  function: (primary_expression
    (subscript
      value: (_) @call.qualifier) @call))

; Parenthesized call target: (f)()
(call
  function: (primary_expression
    (parenthesized_expression) @call))

; Chained/curried call: get_fn()()
(call
  function: (primary_expression
    (call) @call))
