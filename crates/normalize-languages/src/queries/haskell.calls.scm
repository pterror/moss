; Haskell calls query
; @call — call expression nodes
; @call.qualifier — qualifier for qualified calls
;
; Haskell uses juxtaposition for function application: `f x y` is an `apply`
; expression. The function being applied is the first child. There are no
; explicit call parentheses required.

; Function application: f x y
; The `function` field names the applied function (an expression)
(apply
  function: (variable) @call)

; Qualified call: Module.func args
; Qualified references use a single `qualified` node with `module:`/`id:`
; fields (there is no separate `qualified_variable`/`qualified_constructor`
; node type). `module:` wraps a `module_id` leaf whose text has no trailing
; dot, unlike the `module` node's own text.
(apply
  function: (qualified
    module: (module (module_id) @call.qualifier)
    id: (variable) @call))

; Constructor application: Foo x
(apply
  function: (constructor) @call)

; Qualified constructor: Module.Ctor x
(apply
  function: (qualified
    module: (module (module_id) @call.qualifier)
    id: (constructor) @call))

; Operator-section-as-prefix-function application: (+) 1 2, ($) f x.
; `apply.function` allows `prefix_id`, which wraps a bare (unqualified)
; `operator`/`constructor_operator` when a Haskell operator is parenthesized
; and used in prefix (function) position. This is a pervasive idiom — `($)`
; applied directly (e.g. `foldr ($) x fs`) is extremely common — and was
; entirely unmatched before: neither the `variable`/`constructor` clauses
; above nor the `qualified` clauses below cover a bare `prefix_id`.
(apply
  function: (prefix_id
    (operator) @call))
(apply
  function: (prefix_id
    (constructor_operator) @call))

; Qualified operator-section-as-prefix-function application:
; (Prelude.+) 1 2, (Map.!) m k. `prefix_id` also wraps a `qualified` node
; when the operator itself is qualified.
(apply
  function: (prefix_id
    (qualified
      module: (module (module_id) @call.qualifier)
      id: (operator) @call)))
(apply
  function: (prefix_id
    (qualified
      module: (module (module_id) @call.qualifier)
      id: (constructor_operator) @call)))

; Parenthesized-identifier-as-function application: (f) 1, (Map.lookup) k m.
; `apply.function` allows `parens`, whose own `expression` field can hold a
; plain `variable`/`constructor` or a `qualified` reference (redundant
; parens around an ordinary name, used e.g. to disambiguate precedence).
; NOTE: `parens` wrapping an `infix` expression (point-free composition,
; `(f . g) x`) is deliberately NOT matched here — the applied value is the
; composed function, not a single nameable call site, so there is no single
; identifier to attribute the call to.
(apply
  function: (parens
    expression: (variable) @call))
(apply
  function: (parens
    expression: (constructor) @call))
(apply
  function: (parens
    expression: (qualified
      module: (module (module_id) @call.qualifier)
      id: (variable) @call)))
(apply
  function: (parens
    expression: (qualified
      module: (module (module_id) @call.qualifier)
      id: (constructor) @call)))
