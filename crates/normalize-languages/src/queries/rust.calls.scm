; Rust calls query
; @call — call expression nodes (read context)
; @call.write — call expression nodes whose result is being assigned (write context)
; @call.qualifier — qualifier/receiver for method calls

; Simple call: func()
(call_expression
  function: (identifier) @call)

; Scoped call: module::func()
(call_expression
  function: (scoped_identifier
    path: (_) @call.qualifier
    name: (identifier) @call))

; Method call: obj.method()
(call_expression
  function: (field_expression
    value: (_) @call.qualifier
    field: (field_identifier) @call))

; Turbofish call: func::<T>()
(call_expression
  function: (generic_function
    function: (identifier) @call))

; Turbofish scoped call: module::func::<T>()
(call_expression
  function: (generic_function
    function: (scoped_identifier
      path: (_) @call.qualifier
      name: (identifier) @call)))

; Turbofish method call: obj.method::<T>()
(call_expression
  function: (generic_function
    function: (field_expression
      value: (_) @call.qualifier
      field: (field_identifier) @call)))

; Write-context: result of call assigned to a variable or field
; e.g. `bar = baz()`, `*ptr = foo()`, `obj.field = get_val()`
; The @call.write capture replaces @call for these patterns so the
; collect_calls_with_query function can tag access = "write".

; Simple assignment RHS: bar = func()
(assignment_expression
  right: (call_expression
    function: (identifier) @call.write))

; Scoped assignment RHS: bar = module::func()
(assignment_expression
  right: (call_expression
    function: (scoped_identifier
      path: (_) @call.qualifier
      name: (identifier) @call.write)))

; Method assignment RHS: bar = obj.method()
(assignment_expression
  right: (call_expression
    function: (field_expression
      value: (_) @call.qualifier
      field: (field_identifier) @call.write)))

; Turbofish assignment RHS: bar = func::<T>()
(assignment_expression
  right: (call_expression
    function: (generic_function
      function: (identifier) @call.write)))

; Turbofish scoped assignment RHS: bar = module::func::<T>()
(assignment_expression
  right: (call_expression
    function: (generic_function
      function: (scoped_identifier
        path: (_) @call.qualifier
        name: (identifier) @call.write))))

; Turbofish method assignment RHS: bar = obj.method::<T>()
(assignment_expression
  right: (call_expression
    function: (generic_function
      function: (field_expression
        value: (_) @call.qualifier
        field: (field_identifier) @call.write))))

; Compound assignment RHS: x += func(), x -= func()
(compound_assignment_expr
  right: (call_expression
    function: (identifier) @call.write))

(compound_assignment_expr
  right: (call_expression
    function: (scoped_identifier
      path: (_) @call.qualifier
      name: (identifier) @call.write)))

(compound_assignment_expr
  right: (call_expression
    function: (field_expression
      value: (_) @call.qualifier
      field: (field_identifier) @call.write)))

(compound_assignment_expr
  right: (call_expression
    function: (generic_function
      function: (identifier) @call.write)))

(compound_assignment_expr
  right: (call_expression
    function: (generic_function
      function: (scoped_identifier
        path: (_) @call.qualifier
        name: (identifier) @call.write))))

(compound_assignment_expr
  right: (call_expression
    function: (generic_function
      function: (field_expression
        value: (_) @call.qualifier
        field: (field_identifier) @call.write))))
