; Dart calls query
; @call — call expression identifiers
;
; In Dart, function calls appear as: identifier followed by selector(argument_part)
; A selector containing argument_part represents the call.
; The identifier precedes the selector as a sibling in the parent node.

; Simple function call: func()
((identifier) @call
 .
 (selector
   (argument_part)))

; Method call: obj.method(), list.map(...).toList() (chained), list.sort<int>()
; (generic method call — type_arguments live inside the trailing
; argument_part selector, which doesn't change this shape).
;
; Verified via `normalize syntax ast`: `obj.method()` is flat, not nested
; under a call-expression node — `identifier(obj)`, `selector` wrapping
; `unconditional_assignable_selector(. identifier(method))`, then a
; *separate* sibling `selector` wrapping `argument_part`. The previous
; query only matched the bare-identifier-then-selector(argument_part) shape,
; so every method call (the dominant call form in an object-oriented
; language) was silently dropped from extraction — `obj.method()`,
; chained calls, and named-constructor calls like `Point.origin()` all
; produced zero @call captures. `(_ ...)` matches any enclosing node type
; since the common parent varies (block, function_body, ...).
(_
  (selector
    (unconditional_assignable_selector
      (identifier) @call))
  .
  (selector
    (argument_part)))

; Null-aware method call: obj?.method()
(_
  (selector
    (conditional_assignable_selector
      (identifier) @call))
  .
  (selector
    (argument_part)))
