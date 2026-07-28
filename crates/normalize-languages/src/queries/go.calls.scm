; Go calls query
; @call — call expression nodes
; @call.qualifier — qualifier/receiver for method calls
;
; call_expression.function is the `_expression` supertype (~23 grammar
; variants: identifier, selector_expression, call_expression, func_literal,
; index_expression, type_conversion_expression, …). Only identifier and
; selector_expression are handled below; the rest are deliberately excluded
; because @call's text is used downstream as a call-graph name string
; (see normalize-facts/src/symbols.rs collect_calls_with_query) — capturing
; a nameless callee would corrupt that index, not extend it. Verified via
; `normalize syntax query` against real Go idioms:
;   - func_literal (`go func(){}()`, `defer func(){}()` — the idiomatic
;     immediately-invoked-closure form): anonymous callee, no name.
;   - call_expression (curried calls `adder(1)(2)`): callee is a call
;     result, not a named symbol; rare in idiomatic Go.
;   - index_expression (`funcs[0]()`, `handlers[event]()` dispatch tables):
;     callee is a computed collection element, no static name.
;   - Explicit generic-function instantiation (`Sum[int](args)`) does not
;     even parse as call_expression in this grammar version — it parses as
;     type_conversion_expression (a documented tree-sitter-go grammar
;     ambiguity: syntactically identical to converting a value to the
;     generic-instantiated type `Sum[int]`). Non-instantiated generic calls
;     (`Sum(args)`, relying on type inference) parse as ordinary
;     call_expression/identifier and are already covered below.

; Simple call: func()
(call_expression
  function: (identifier) @call)

; Method/package call: obj.method() or pkg.Func()
(call_expression
  function: (selector_expression
    operand: (_) @call.qualifier
    field: (field_identifier) @call))
