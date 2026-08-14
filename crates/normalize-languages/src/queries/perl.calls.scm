; Perl calls query
; @call — call expression nodes
; @call.qualifier — qualifier/receiver for method calls

; Simple function call, parenthesized: func(args)
(function_call_expression
  function: (function) @call)

; Simple function call, parenless with at least one argument: func args —
; the grammar uses a structurally distinct node type
; (ambiguous_function_call_expression, so named because the parser can't
; tell at parse time whether a bareword-plus-list is a call or a list
; literal) for this form. Both node types constrain their `function` field
; to the same `function` node type. Verified via `normalize syntax query`:
; `print "hi $name\n";` and a parenless user-sub call (`mysub $name, 1;`)
; both parse as ambiguous_function_call_expression, not
; function_call_expression — without this clause, every parenless call
; with arguments (print/warn/every bareword-style sub invocation, extremely
; common Perl idiom) was silently dropped from @call entirely.
;
; A bareword with NO trailing arguments and no parens (`foo;` alone) is a
; genuine grammar limitation, not a missing clause here: it parses as a
; bare `bareword` expression statement, not as any call_expression node at
; all (verified via `normalize syntax ast`) — the grammar can't
; disambiguate a zero-argument bareword from a plain string/list constant
; without a following argument list, so there is nothing to capture.
(ambiguous_function_call_expression
  function: (function) @call)

; Method call: $obj->method(args) or Class->method(args)
(method_call_expression
  invocant: (_) @call.qualifier
  method: (method) @call)
