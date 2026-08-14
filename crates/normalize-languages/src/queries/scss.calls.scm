; SCSS calls query
; @call — function call expression nodes
; @call.qualifier — not applicable (SCSS functions are not method calls)
;
; SCSS function calls appear as `call_expression` nodes with a `function_name`
; child (the callee) and an `arguments` child. Examples:
;   rgba(255, 0, 0, 0.5)
;   darken($color, 10%)
;   map-get($map, key)

; Function call: func(args...)
(call_expression
  (function_name) @call)

; Mixin invocation: @include mixin-name; / @include mixin-name(args);
; `include_statement` has no dedicated call_expression wrapper — the mixin
; name is a direct `identifier` child (confirmed via `normalize syntax
; ast`), structurally distinct from a function call but semantically a call
; (it invokes the mixin's body). @include is one of the two idioms (with
; @mixin itself) that define SCSS's primary code-reuse mechanism, so
; dropping it here would silently miss the majority of "calls" in any
; mixin-heavy stylesheet.
(include_statement
  (identifier) @call)
