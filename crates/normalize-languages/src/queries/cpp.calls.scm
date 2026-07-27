; C++ calls query
; @call — call expression nodes
; @call.qualifier — qualifier/receiver for method calls

; Simple call: func()
(call_expression
  function: (identifier) @call)

; Field/pointer member call: obj.method() or ptr->method()
(call_expression
  function: (field_expression
    argument: (_) @call.qualifier
    field: (field_identifier) @call))

; Template method call: obj.method<T>() / ptr->method<T>() — `field` is
; `template_method`, not `field_identifier`; unhandled meant every explicit
; template-argument method call (extremely common on generic containers,
; e.g. `tuple.get<0>()`) was silently dropped.
(call_expression
  function: (field_expression
    argument: (_) @call.qualifier
    field: (template_method) @call))

; Explicit destructor call: obj.~Foo()
(call_expression
  function: (field_expression
    argument: (_) @call.qualifier
    field: (destructor_name) @call))

; Explicit base-class-qualified member call: obj.Base::method() — `field` is
; a nested `qualified_identifier`, disambiguating which base's override to
; invoke.
(call_expression
  function: (field_expression
    argument: (_) @call.qualifier
    field: (qualified_identifier) @call))

; Qualified/namespace call: Ns::func() or Class::method()
(call_expression
  function: (qualified_identifier
    scope: (_) @call.qualifier
    name: (identifier) @call))

; Scoped template-argument call: std::get<0>(x), ns::helper<int>() — `name`
; is `template_function`, not `identifier`.
(call_expression
  function: (qualified_identifier
    scope: (_) @call.qualifier
    name: (template_function) @call))

; Plain template-argument call: identity<int>(5) — direct C++ analogue of
; Rust's turbofish gap (docs/query-testing-methodology.md): `function` is
; `template_function` directly, with no qualifier/field wrapper at all. A
; routine idiom for any generic function called with explicit template
; arguments (std::make_shared<T>(), std::static_pointer_cast<T>(), etc. all
; parse this way when unqualified via a `using` or ADL).
(call_expression
  function: (template_function) @call)
