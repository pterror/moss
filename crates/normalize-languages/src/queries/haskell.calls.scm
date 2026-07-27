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
