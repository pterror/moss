; C# calls query
; @call — call expression nodes
; @call.qualifier — qualifier/receiver for method calls
;
; `invocation_expression.function` is the `expression` supertype (a very
; large set — every C# expression kind). Only a handful of its variants are
; ever actually a callee shape in practice: plain `identifier` (unqualified
; call), `generic_name` (unqualified generic call, e.g. a local generic
; method `Bar<int>()`), and `member_access_expression` (qualified call,
; `obj.Method()`) — whose own `name:` field is itself `[generic_name
; identifier]`, so a qualified GENERIC call (`list.OfType<T>()`,
; `Enumerable.Empty<T>()` — an extremely common LINQ idiom) needs the same
; generic_name handling nested one level deeper. Verified against real parse
; output via `normalize syntax query`; every one of these was silently
; dropped before this fix except the two plain-identifier forms.

; Simple invocation: Method()
(invocation_expression
  function: (identifier) @call)

; Simple generic invocation: Method<T>()
(invocation_expression
  function: (generic_name (identifier) @call))

; Member access invocation: obj.Method() / obj.Method<T>()
(invocation_expression
  function: (member_access_expression
    expression: (_) @call.qualifier
    name: [(identifier) @call (generic_name (identifier) @call)]))

; Null-conditional member access invocation: obj?.Method() / obj?.Method<T>()
; and chained obj?.A()?.B(). `?.` parses as a distinct `conditional_access_
; expression` (condition + a `member_binding_expression`/`element_binding_
; expression` child), NOT a `member_access_expression` — so it was entirely
; unmatched by the pattern above despite being a near-universal C# null-
; safety idiom.
(invocation_expression
  function: (conditional_access_expression
    condition: (_) @call.qualifier
    (member_binding_expression
      name: [(identifier) @call (generic_name (identifier) @call)])))

; Explicit constructor invocation: base(...)/this(...) constructor
; delegation. `constructor_initializer` is a distinct node kind (not
; `invocation_expression`) whose `base`/`this` keyword is an anonymous
; string token, not a named child or field — so every subclass constructor
; delegating to its base class (or to a sibling overload via `this(...)`)
; silently disappeared from call extraction before this fix.
(constructor_initializer "base" @call)

(constructor_initializer "this" @call)
