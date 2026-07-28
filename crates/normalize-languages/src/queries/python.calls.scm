; Python calls query
; @call — call expression nodes
; @call.qualifier — qualifier/receiver for method calls
;
; `call.function` (per arborium-python's node-types.json) allows the full
; `primary_expression` variant set: attribute, await, binary_operator, call,
; concatenated_string, dictionary, dictionary_comprehension, ellipsis,
; false, float, generator_expression, identifier, integer, list,
; list_comprehension, list_splat, none, parenthesized_expression, set,
; set_comprehension, string, subscript, true, tuple, unary_operator.
; Only `identifier`, `attribute`, and `subscript` ever produce a
; meaningful, nameable callee in real code (verified via `normalize syntax
; ast`/`normalize syntax query` against probe files); the rest are
; syntactically legal but semantically nonsensical (e.g. calling an
; integer literal) and never occur in practice, so they're intentionally
; left unhandled rather than adding dead query clauses.
;
; `call` as `function` (chained calls, e.g. `get_func()()`) needs no
; separate clause: the inner call (`get_func()`) is itself a `call` node
; elsewhere in the tree and is already matched by the plain-identifier
; rule below. `parenthesized_expression` as `function` (immediately
; invoked lambdas/conditionals, e.g. `(lambda x: x)(5)`) is intentionally
; unhandled — there is no static name to extract, and fabricating one
; would misrepresent the callee.

; Simple call: func()
(call
  function: (identifier) @call)

; Method call: obj.method()
(call
  function: (attribute
    object: (_) @call.qualifier
    attribute: (identifier) @call))

; Subscript-dispatched call: handlers["key"](), TABLE[i](), a.b["k"]()
; (dict/list-dispatch idiom — command pattern, event routing tables).
; There is no statically-resolved callee name here (the same limitation
; applies to `f()` where `f` is just a variable holding a callable); by
; the same convention we report the subscripted container's name as a
; best-effort approximation rather than nothing.
(call
  function: (subscript
    value: [
      (identifier) @call
      (attribute attribute: (identifier) @call)
    ]))
