; Scala calls query
; @call — call expression nodes
; @call.qualifier — qualifier/receiver for method calls

; Simple call: func()
(call_expression
  function: (identifier) @call)

; Method call: obj.method()
(call_expression
  function: (field_expression
    value: (_) @call.qualifier
    field: (identifier) @call))

; Explicit operator-method call: obj.+(x), this.n.+(1) — `field_expression.field`
; allows `operator_identifier` in addition to `identifier` (confirmed via real
; parse output: `this.n.+` parses with field = operator_identifier "+").
(call_expression
  function: (field_expression
    value: (_) @call.qualifier
    field: (operator_identifier) @call))

; Generic/type-parameterized call: func[T]()
(call_expression
  function: (generic_function
    function: (identifier) @call))

; Qualified generic call: obj.method[T]()
(call_expression
  function: (generic_function
    function: (field_expression
      value: (_) @call.qualifier
      field: (identifier) @call)))

; Parenthesized call target: (f)(x). `call_expression.function` allows
; `parenthesized_expression` directly (confirmed via `normalize syntax
; query`); the whole parenthesized text is captured, mirroring
; typescript.calls.scm's treatment of the same shape.
(call_expression
  function: (parenthesized_expression) @call)
