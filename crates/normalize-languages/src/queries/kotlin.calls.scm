; Kotlin calls query
; @call — call expression nodes
; @call.qualifier — qualifier/receiver for method calls

; Simple call: func()
(call_expression
  (simple_identifier) @call)

; Member/navigation call: obj.method()
(call_expression
  (navigation_expression
    (_) @call.qualifier
    (navigation_suffix
      (simple_identifier) @call)))

; Explicit constructor delegation: `this(...)` / `super(...)` inside a
; secondary constructor. `constructor_delegation_call` is a distinct node
; kind from `call_expression` (its only named child is `value_arguments`;
; "this"/"super" are anonymous keyword tokens) — entirely unmatched
; before, silently dropping every secondary-constructor delegation.
(constructor_delegation_call
  ("this") @call)

(constructor_delegation_call
  ("super") @call)
