; VimScript calls query
; @call — function call expression
; @call.qualifier — not applicable
;
; VimScript has two forms of function calls:
; 1. Expression calls: `Foo()`, `s:Foo()` — `call_expression` nodes with a
;    `function` field that is either an `identifier` or `scoped_identifier`.
; 2. Statement calls: `call Foo()` — `call_statement` wraps a `call_expression`.
;    The inner `call_expression` is matched by the rules below.
;
; node-types.json's `call_expression.function` field allows a much larger
; set of expression node types (binary_operation, ternary_expression,
; lambda_expression, method_expression, …) since the grammar reuses the
; generic expression production for that field. Most of those never
; actually arise as a call target in practice — verified via
; `normalize syntax query` against `->` method-chain probes
; (`range(5)->map(...)`, `a->b()->c()`): the chain's individual calls
; (`map`, `b`, `c`) parse as plain `identifier` function fields nested
; inside the method_expression, not as a call whose function is itself a
; method_expression — so no separate clause is needed for that variant.
; `field_expression` and `index_expression`, however, are real and
; distinct: dict-bound method calls (`g:dict.Func()`) and dynamic
; dispatch-table calls (`d['Func']()`).

; Simple function call: Foo()
(call_expression
  function: (identifier) @call)

; Scoped function call: s:Foo(), g:Bar(), etc.
(call_expression
  function: (scoped_identifier
    (identifier) @call))

; Dict-bound method call: g:dict.Func(), obj.Method()
(call_expression
  function: (field_expression) @call)

; Dynamic dispatch-table call: d['Func'](), dict[key]()
(call_expression
  function: (index_expression) @call)
