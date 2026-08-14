; Julia calls query
; @call — call expression nodes
; @call.qualifier — qualifier/receiver for method calls
;
; call_expression/broadcast_call_expression have no named fields (node-types.json:
; empty fields object) — the callee is whichever positional child precedes the
; argument_list. Cross-referenced against node-types.json's full children list
; and verified per-variant via `normalize syntax ast`/`syntax query` against a
; probe file: identifier, field_expression (already handled), and three
; previously-missing variants — parametrized_type_expression (`Vector{Int}(undef, 3)`,
; the idiomatic way to construct a parametric type), index_expression
; (`handlers[1](5)`, calling the result of an index/dispatch-table lookup), and
; parenthesized_expression (`(x -> x*2)(10)`, IIFE-style immediate invocation of
; an anonymous function) — were silently unmatched.
;
; KNOWN LIMITATION, not fixed here: this grammar reuses call_expression as the
; parameter-list head of both traditional definitions (`function foo(x) ... end`,
; where the call_expression is nested directly under a `signature` node) and
; short-form definitions (`foo(x) = ...`, where the call_expression is the
; literal first child of an `assignment`). The short-form case can be (and is,
; in julia.tags.scm) distinguished structurally with a `.` first-child anchor,
; but the `signature`-wrapped case cannot: this query engine only supports
; eq?/not-eq?/match?/not-match?/any-of? predicates (see
; normalize-syntax-rules::runner::evaluate_predicates) — there is no
; ancestor/parent-type predicate anywhere in this codebase, and tree-sitter
; query matches from independent patterns union rather than subtract, so a
; more specific pattern cannot suppress a broader one's match on the same
; node. Confirmed via `normalize syntax query` that every `function foo(x)
; ... end` definition currently also emits a bogus @call = "foo". This is the
; identical class of limitation already documented (and deliberately left
; unfixed) in commonlisp.calls.scm's "a single flat tree-sitter pattern cannot
; express [parent context]" note — not a gap to special-case away.

; Simple call: func(args)
(call_expression
  (identifier) @call)

; Method call: obj.method(args) — field_expression as callee
(call_expression
  (field_expression
    (_) @call.qualifier
    (identifier) @call))

; Parametric-type constructor call: Vector{Int}(undef, 3), Dict{String,Any}()
(call_expression
  (parametrized_type_expression
    (identifier) @call))

; Call on an indexed/dispatch-table result: handlers[1](5)
(call_expression
  (index_expression) @call)

; Call on a parenthesized expression (IIFE-style): (x -> x*2)(10)
(call_expression
  (parenthesized_expression) @call)

; Broadcast call: func.(args) — vectorized application
(broadcast_call_expression
  (identifier) @call)

; Broadcast method call: obj.method.(args)
(broadcast_call_expression
  (field_expression
    (_) @call.qualifier
    (identifier) @call))

; Broadcast parametric-type constructor call: Vector{Int}.(sizes)
(broadcast_call_expression
  (parametrized_type_expression
    (identifier) @call))

; Broadcast call on an indexed result: handlers[1].(args)
(broadcast_call_expression
  (index_expression) @call)

; Broadcast call on a parenthesized expression: (x -> x*2).(values)
(broadcast_call_expression
  (parenthesized_expression) @call)
