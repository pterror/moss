; F# calls query
; @call — call expression nodes
; @call.qualifier — qualifier/receiver for method calls
;
; F# uses `application_expression` for function application (juxtaposition):
; `f x y` is an application_expression. Method calls use dot access.
;
; `application_expression` has no named fields at all (grammar declares
; `"fields": {}`) — the applied function is just the first positional child,
; so patterns below anchor on `.` instead of a `function:` field. The field
; on `dot_expression` for the member name is `field:`, and its type is
; `long_identifier_or_op` (not `long_identifier`).
;
; Note: the grammar's precedence handling for application vs. infix
; expressions is ambiguous in some cases (e.g. `n * factorial (n - 1)` parses
; with an `infix_expression` as the application's first child rather than the
; function identifier), so a recursive call written that way is not captured
; here — that's a CST limitation, not a query bug.

; Function application: f x y
(application_expression
  .
  (long_identifier_or_op (identifier) @call .))

; Qualified call: Module.func args
(application_expression
  .
  (long_identifier_or_op
    (long_identifier
      (identifier) @call.qualifier
      (identifier) @call .)))

; Method call: obj.Method(args)
(dot_expression
  base: (_) @call.qualifier
  field: (long_identifier_or_op
    (identifier) @call))
