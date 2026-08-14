; ReScript calls query
; @call — call expression
; @call.qualifier — module/record qualifier for qualified calls
;
; ReScript (formerly BuckleScript/Reason) has ML-style function calls.
; `call_expression.function` is typed `primary_expression`, a large
; supertype (array, call_expression, member_expression, parenthesized_
; expression, pipe_expression, value_identifier, value_identifier_path,
; variant, ...) — cross-checked against arborium-rescript-2.17.0's
; node-types.json. Two forms account for the vast majority of real call
; sites and are handled below; the remaining subtypes (calling the result
; of an if/switch/try expression, an immediately-invoked lambda, etc.) are
; rare enough in practice not to be worth a dedicated pattern, and have no
; single static name to report as @call anyway.

; Simple call: func(args...)
(call_expression
  function: (value_identifier) @call)

; Module-qualified call: Module.func(args...) or Module.Sub.func(args...).
; `value_identifier_path`'s non-final child is `module_primary_expression`
; (itself `module_identifier`/`module_identifier_path`/etc. depending on
; path depth) — anchored `.` so @call.qualifier only ever binds that
; prefix, not the final `(value_identifier)` @call itself.
; @call.qualifier was documented in this file's header but never actually
; captured anywhere — qualified calls (the majority of real ReScript call
; sites, e.g. `arr->Belt.Array.map(f)`) silently lost their qualifier.
(call_expression
  function: (value_identifier_path
    .
    (_) @call.qualifier
    (value_identifier) @call))

; Record/object field call: obj.field(args...) — `member_expression` was
; entirely unhandled; a call whose function is a record field (a common
; idiom for `Js.t`/bindings-style objects) was silently dropped from
; extraction. Verified via `normalize syntax ast`/`normalize syntax query`.
(call_expression
  function: (member_expression
    record: (_) @call.qualifier
    property: (property_identifier) @call))

; KNOWN LIMITATION (grammar, not this query): a paren-less pipe application
; of a single-argument function, e.g. `xs->Belt.Array.length` (semantically
; a call — ReScript desugars this to `Belt.Array.length(xs)`), parses as a
; bare `value_identifier_path` directly as `pipe_expression`'s right
; operand, NOT wrapped in a `call_expression` at all — verified via
; `normalize syntax ast`. There is no call_expression node here for any
; pattern to match, so this idiom is invisible to @call extraction. Only
; the explicit-parens pipe form (`xs->Belt.Array.map(f)`, handled above via
; the ordinary call_expression pattern once the parser wraps it) is
; captured.
