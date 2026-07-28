; Swift calls query
; @call — call expression nodes
; @call.qualifier — qualifier/receiver for method calls

; Simple call: func()
; call_expression has no fields — children are the callee and call_suffix
(call_expression
  (simple_identifier) @call
  (call_suffix))

; Member/navigation call: obj.method(), self.method(), super.method()
; (target: (_) matches any qualifier expression, including self/super).
(call_expression
  (navigation_expression
    target: (_) @call.qualifier
    suffix: (navigation_suffix
      (simple_identifier) @call))
  (call_suffix))

; Force-unwrap call: `completion!()` — calling an optional closure/function
; value after force-unwrapping it. Verified via `normalize syntax ast`: the
; callee is a postfix_expression (target/operation fields) sitting directly
; inside call_expression, a sibling of call_suffix — a distinct shape from
; both patterns above, and previously unmatched by either.
(call_expression
  (postfix_expression
    target: (simple_identifier) @call
    operation: (bang))
  (call_suffix))

; Generic type instantiation call: `Array<Int>()`, `Optional<String>(nil)`.
; Verified via `normalize syntax ast`: unlike a non-generic explicit
; initializer call (`Foo()`, which parses as an ordinary call_expression with
; a plain simple_identifier callee — already covered by the first pattern
; above), a call with explicit generic type arguments parses as a DISTINCT
; node type, constructor_expression, with a `constructed_type` field —
; invisible to call_expression-only patterns. This is the same class of bug
; as Rust's turbofish-call gap found in batch 1.
(constructor_expression
  constructed_type: (user_type
    (type_identifier) @call))

; call_expression's callee position (~50 grammar-legal variants) also allows
; several forms with no stable, nameable callee; deliberately NOT matched,
; verified via `normalize syntax ast` against real idioms:
;   - lambda_literal (immediately-invoked closure: `{ ... }(args)`): the
;     callee is an anonymous closure body; capturing its source text as
;     @call would put multi-line closure bodies into the call-graph name
;     index, corrupting it. Matches go.calls.scm's identical exclusion for
;     `go func(){}()`/`defer func(){}()`.
;   - call_expression (curried calls: `adder(1)(2)`, `outer()()`): the callee
;     is the *result* of a call, not a named symbol.
;   - array_type/dictionary_type literal-type initializer calls
;     (`[Int](repeating: 0, count: 3)`, `[String: Int](minimumCapacity: 5)`):
;     the callee is a bracket type-literal, not a declared symbol name.
