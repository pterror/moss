; TSX calls query
; @call — call expression nodes
; @call.qualifier — qualifier/receiver for method calls
;
; call_expression.function is grammar-typed as the `expression` supertype (per
; arborium-tsx's node-types.json), which expands through `primary_expression`
; to ~20 node kinds. The clauses below cover every variant confirmed to
; actually occur as a direct call target via `normalize syntax
; query`/`normalize syntax ast` on real TSX source (see
; docs/query-testing-methodology.md) — ported from typescript.calls.scm,
; which TSX's grammar is otherwise identical to for non-JSX constructs.
; `instantiation_expression` (an explicit generic reference with no call,
; e.g. `const f = identity<number>;`) never appears directly as
; `call_expression.function` in practice — only nested inside a
; `parenthesized_expression`, which the parenthesized-call clause below
; already captures as a whole.

; Simple call: func()
(call_expression
  function: (identifier) @call)

; Method call: obj.method() / obj.#privateMethod()
(call_expression
  function: (member_expression
    object: (_) @call.qualifier
    property: [(property_identifier) (private_property_identifier)] @call))

; Computed/bracket call: obj['method'](), obj[key]()
(call_expression
  function: (subscript_expression
    object: (_) @call.qualifier) @call)

; Parenthesized call target: (foo)(), (identity<number>)()
(call_expression
  function: (parenthesized_expression) @call)

; TS non-null assertion before call: foo!()
(call_expression
  function: (non_null_expression) @call)

; Chained/curried call: connect(mapStateToProps)(Component)
(call_expression
  function: (call_expression) @call)
