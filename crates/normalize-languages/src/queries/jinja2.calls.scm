; Jinja2 calls query
; @call           — the called function/filter/test name
; @call.qualifier — qualifier/receiver for method-style calls
;
; Jinja2 has three call-like constructs:
;   1. Function calls: {{ func(args) }}, {{ obj.method(args) }}
;   2. Filter calls:   {{ value | filter_name }}, {{ value | filter(arg) }}
;   3. Test calls:     {% if value is test_name %}
;
; Additionally, {% call macro_name() %} invokes a macro with a caller block.

; Function call: func(args)
(call_expression
  function: (identifier) @call)

; Method call: obj.method(args)
(call_expression
  function: (attribute_expression
    object: (_) @call.qualifier
    attribute: (identifier) @call))

; Filter application: value | filter_name or value | filter_name(args)
; Each filter_item in a filter chain is a call
(filter_item
  name: (identifier) @call)

; Test expression: value is test_name
(test_expression
  test: (identifier) @call)

; Call statement: {% call macro_name(args) %}...{% endcall %}
; The callee is an expression — capture its function name
(call_statement
  callee: (call_expression
    function: (identifier) @call))

(call_statement
  callee: (call_expression
    function: (attribute_expression
      object: (_) @call.qualifier
      attribute: (identifier) @call)))

; `call_expression.function` and `call_statement.callee` are grammar-typed as
; the full 23-variant expression union (node-types.json), not just
; `identifier`/`attribute_expression`. The remaining 21 variants are all
; computed-callable forms confirmed to actually parse this way — e.g.
; `funcs["key"]()` is `function: (subscript_expression)`, `get_func()()` is
; `function: (call_expression)` (verified via `normalize syntax query`). They
; are deliberately not captured here: none of them carry an `identifier` a
; @call capture could report as a call *name* — the "name" would be an
; arbitrary sub-expression's source text, not a symbol, so a query clause for
; them would produce a capture with no name-like content rather than
; meaningful extraction. `object: (_)` above already covers arbitrarily deep
; attribute chains (a.b.c()) since it is unconstrained.
