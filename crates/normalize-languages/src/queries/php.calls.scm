; PHP calls query
; @call — call expression nodes
; @call.qualifier — qualifier/receiver for method calls

; Simple function call: func(args)
(function_call_expression
  function: (name) @call)

; Variable function call: $func(args)
(function_call_expression
  function: (variable_name
    (name) @call))

; Namespaced function call: \App\Utils\helper(args), Ns\func(args). Also
; covers the remaining computed-callee variants `function_call_expression.
; function` allows (verified via `normalize syntax ast`/`normalize syntax
; query` against real constructs): IIFEs `(function(){})()`
; (parenthesized_expression), callable-array syntax `['Class','method']()`
; (array_creation_expression), subscript access `$arr['key']()`
; (subscript_expression), and chained call/`new` results
; (function_call_expression/member_call_expression/
; nullsafe_member_call_expression/scoped_call_expression/
; object_creation_expression as the callee, e.g. `getCallback()()`). These
; rarer forms are captured wholesale (the call site is never silently
; dropped) even though the captured text isn't always a clean function name.
(function_call_expression
  function: [
    (qualified_name)
    (relative_name)
    (array_creation_expression)
    (dynamic_variable_name)
    (encapsed_string)
    (function_call_expression)
    (heredoc)
    (member_call_expression)
    (nowdoc)
    (nullsafe_member_call_expression)
    (object_creation_expression)
    (parenthesized_expression)
    (scoped_call_expression)
    (string)
    (subscript_expression)
  ] @call)

; Static method call: Class::method(args), Class::$dynamicMethod(args),
; self::{$expr}(args). `scoped_call_expression.name` allows exactly these
; four variants; `expression` is a real tree-sitter supertype here (verified
; it binds to the concrete node, e.g. `binary_expression`, and does NOT also
; match name/variable_name/dynamic_variable_name, so no double-count risk).
(scoped_call_expression
  scope: (_) @call.qualifier
  name: [
    (name)
    (variable_name)
    (dynamic_variable_name)
    (expression)
  ] @call)

; Instance method call: $obj->method(args), $obj->$method(args),
; $obj->{$expr}(args). Same four-variant field shape as scoped_call_expression.
(member_call_expression
  object: (_) @call.qualifier
  name: [
    (name)
    (variable_name)
    (dynamic_variable_name)
    (expression)
  ] @call)

; Nullsafe method call: $obj?->method(args), etc. Same field shape again.
(nullsafe_member_call_expression
  object: (_) @call.qualifier
  name: [
    (name)
    (variable_name)
    (dynamic_variable_name)
    (expression)
  ] @call)

; Constructor invocation (`new Foo()`) is intentionally NOT treated as a call
; here, matching java.calls.scm's precedent: it's a class reference
; (php.tags.scm's @reference.class), not a function/method invocation.
