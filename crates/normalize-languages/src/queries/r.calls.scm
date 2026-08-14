; R calls query
; @call — call expression nodes
; @call.qualifier — qualifier/receiver for namespace-qualified or method calls
;
; In R's tree-sitter grammar, function calls are `call` nodes. `call.function`
; allows many expression types (per node-types.json); only the ones below are
; realistic call-target shapes. `call` itself (`f()()`), literals, and
; control-flow nodes as `function` are syntactically legal but never occur in
; real code, so they're intentionally not matched.

; Simple call: func(args)
(call
  function: (identifier) @call)

; Namespace-qualified call: pkg::func(args) or pkg:::func(args)
; The node type is `namespace_operator` with `lhs:`/`rhs:` fields, not
; `namespace_get` with `namespace:`/`function:`.
(call
  function: (namespace_operator
    lhs: (_) @call.qualifier
    rhs: (identifier) @call))

; Method-style call via `$`: obj$method(args) or self$method(args).
; `extract_operator` is the `$`/`@` (slot) accessor node; `rhs` is the
; method-name identifier. Extremely common in R6/Reference-Class/
; environment-based OOP (self$method(), private$run()).
(call
  function: (extract_operator
    lhs: (_) @call.qualifier
    rhs: (identifier) @call))

; Method-style call via `[[`: obj[["method"]](args). `subset2.function` is
; the base object; the callee name lives inside `arguments` (typically a
; string), not as a directly-named field, so only the qualifier is captured
; here — the call site itself is still marked via the outer `call` capture
; on the whole subset2 node (kind distinguishes it from an identifier call).
(call
  function: (subset2
    function: (_) @call.qualifier) @call)

; Method-style call via `[`: obj["method"](args). Same rationale as the
; subset2 case above (single-bracket form).
(call
  function: (subset
    function: (_) @call.qualifier) @call)
