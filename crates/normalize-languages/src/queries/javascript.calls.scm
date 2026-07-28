; JavaScript calls query
; @call — call expression nodes
; @call.qualifier — qualifier/receiver for method calls
;
; call_expression.function is grammar-typed as the `expression` supertype (per
; arborium-javascript's node-types.json), which expands through
; `primary_expression` to ~20 node kinds. The clauses below cover every
; variant confirmed to actually occur as a direct call target via `normalize
; syntax query`/`normalize syntax ast` on real JS source (see
; docs/query-testing-methodology.md).

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

; Parenthesized call target: (function(){})()  (IIFE)
(call_expression
  function: (parenthesized_expression) @call)

; Chained/curried call: connect(mapStateToProps)(Component)
(call_expression
  function: (call_expression) @call)
