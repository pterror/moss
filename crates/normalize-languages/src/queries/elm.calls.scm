; Elm calls query
; @call — function application nodes
; @call.qualifier — module qualifier for qualified calls
;
; Elm is a purely functional language using juxtaposition for application.
; `f x y` is a `function_call_expr` with a `target` field for the function
; and `arg` fields for arguments.
;
; `function_call_expr.target` allows five node kinds per node-types.json
; (verified via `normalize syntax ast`/`normalize syntax query`):
; `value_expr` (the plain/qualified/constructor cases below),
; `field_access_expr` (calling a function reached through a record field,
; e.g. `model.update msg` — a real, if less common, TEA-dispatch-table
; idiom), `field_accessor_function_expr` (calling a bare `.field` accessor
; directly, e.g. `.name record`), `operator_as_function_expr` (calling an
; operator used as a function, e.g. `(+) 1 2`), and `parenthesized_expr`
; (currying/partial application written with explicit parens, e.g.
; `(f) x`). All five are handled below; previously only `value_expr` was.

; Simple function application: f x
(function_call_expr
  target: (value_expr
    name: (value_qid
      (lower_case_identifier) @call)))

; Module-qualified call: Module.func x
(function_call_expr
  target: (value_expr
    name: (value_qid
      (upper_case_identifier) @call.qualifier
      (lower_case_identifier) @call)))

; Constructor application: Foo x (uppercase constructors)
(function_call_expr
  target: (value_expr
    name: (upper_case_qid
      (upper_case_identifier) @call)))

; Call through a record field: model.update msg (the field path is
; captured whole — statically indistinguishable from an ordinary field
; access without type information).
(function_call_expr
  target: (field_access_expr) @call)

; Call of a bare field-accessor function used directly: .name record
(function_call_expr
  target: (field_accessor_function_expr) @call)

; Call of an operator used as a function: (+) 1 2
(function_call_expr
  target: (operator_as_function_expr) @call)

; Parenthesized target wrapping a plain identifier/qualified/constructor
; call, e.g. `(f) x`, `(Module.f) x`, `(Just) x` — unwraps one level of
; `parenthesized_expr` to the same three shapes as above. A
; `parenthesized_expr` wrapping something else (e.g. `(f x) y`, itself
; already a `function_call_expr` matched independently above) is left
; uncaptured at this outer application — the callee there isn't a single
; static name, it's the *result* of the inner call.
(function_call_expr
  target: (parenthesized_expr
    expression: (value_expr
      name: (value_qid
        (lower_case_identifier) @call))))

(function_call_expr
  target: (parenthesized_expr
    expression: (value_expr
      name: (value_qid
        (upper_case_identifier) @call.qualifier
        (lower_case_identifier) @call))))

(function_call_expr
  target: (parenthesized_expr
    expression: (value_expr
      name: (upper_case_qid
        (upper_case_identifier) @call))))
