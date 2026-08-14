; Gleam calls query
; @call — call expression nodes
; @call.qualifier — qualifier/receiver for module-qualified calls

; Simple call: func(args)
(function_call
  function: (identifier) @call)

; Module-qualified call: module.func(args)
(function_call
  function: (field_access
    record: (_) @call.qualifier
    field: (label) @call))

; Point-free pipe target: x |> f (no parens — `f` is invoked with `x` as its
; sole argument via the pipe operator, but the grammar does not wrap it in a
; function_call node, so it's invisible to the two patterns above). Verified
; via `normalize syntax ast`: `x |> double` parses as
; `binary_expression(left: identifier, operator: "|>", right: identifier)`
; with no function_call wrapping the right side. This is idiomatic Gleam
; (pipelines are the dominant call style) and arborium's own highlights.scm
; special-cases exactly this shape for the same reason.
(binary_expression
  operator: "|>"
  right: (identifier) @call)

; Point-free *qualified* pipe target: x |> module.func (no call parens on
; the right side either). A separate gap from the bare-identifier case above
; — `right: (field_access ...)` with no wrapping function_call. Verified via
; `normalize syntax ast`/`normalize syntax query` against
; `values |> list.length` in variants.gleam.
(binary_expression
  operator: "|>"
  right: (field_access
    record: (_) @call.qualifier
    field: (label) @call))

; NOTE: `function_call.function` also allows `function_call` (curried
; application, `add(1)(2)`) and `tuple_access` (`#(f, g).0(5)`) per
; node-types.json. Both are syntactically legal but not idiomatic Gleam
; (confirmed absent from arborium's own highlights.scm and from the
; wisp/router samples); the inner named call is still captured via the
; patterns above, only the outer application node itself (which has no
; identifier to name) is not. Left undocumented as a capture since there is
; no meaningful name to attach to it.
