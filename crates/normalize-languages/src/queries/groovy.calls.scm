; Groovy calls query
; @call — call expression nodes
; @call.qualifier — qualifier/receiver for method calls
;
; Completeness matrix verified against arborium-groovy 2.17.0's
; node-types.json via `normalize syntax ast`/`normalize syntax query`.
; `function_call.function` allows: closure, dotted_identifier, function_call,
; identifier, index, parenthesized_expression, this. `juxt_function_call.function`
; allows: dotted_identifier, identifier, index.
;
; `closure` as `function_call.function` (an IIFE like `{ -> ... }()`) is listed
; as legal in node-types.json but is NOT reachable in practice: the grammar
; parses `{ -> println "hi" }()` as two separate top-level statements (a bare
; closure, then a separate `()` parenthesized expression), not as one
; function_call — confirmed via `normalize syntax ast`. No query clause is
; added for it (per docs/query-testing-methodology.md step 3: node-types.json
; occasionally lists positions the grammar never actually produces).

; Regular function/method call: func(args)
(function_call
  function: (identifier) @call)

; Juxtaposition call: method arg (Groovy allows no-parens calls)
(juxt_function_call
  function: (identifier) @call)

; Method call with receiver: obj.method(args)
(function_call
  function: (dotted_identifier
    (_) @call.qualifier
    (identifier) @call))

; Juxtaposition call with a dotted receiver: Math.max 1, 2
;
; Reliable in a small, isolated snippet (confirmed via `normalize syntax
; ast`/`normalize syntax query`), but arborium-groovy 2.17.0 appears to have
; a genuine GLR ambiguity-resolution artifact that makes this specific shape
; unreliable once embedded in a larger file: the same construct silently
; reparses as two separate sibling statements (dotted_identifier, then a
; bare identifier — no ERROR node) depending on surrounding parse context,
; independent of local factors like adjacent comments or statement
; terminators (verified by bisecting a real fixture's prefix length: pass/
; fail toggles non-monotonically). The clause is kept because the shape is
; real and does fire for at least some inputs, but it should not be assumed
; to fire reliably on arbitrary real-world Groovy files with this grammar
; version. See tests/fixtures/groovy/variants.groovy for the full writeup.
(juxt_function_call
  function: (dotted_identifier
    (_) @call.qualifier
    (identifier) @call))

; Chained/curried call: curry()()
(function_call
  function: (function_call) @call)

; Bracket-index dispatch-table call: handlers['create'](x)
; (only reachable as a declaration/assignment RHS or nested expression — a
; bare top-level `handlers['create'](x)` statement is ambiguously parsed as
; two separate statements by this grammar; see groovy.tags.scm note.)
(function_call
  function: (index) @call)

; Bracket-index dispatch-table call, no-parens form: handlers['create'] x
;
; Shares the same context-dependent unreliability as the dotted-receiver
; juxtaposition clause above — see that clause's comment.
(juxt_function_call
  function: (index) @call)

; Ternary-selected call: (cond ? f1 : f2)(x)
(function_call
  function: (parenthesized_expression) @call)

; this(args) call form. `this` is an anonymous token in this grammar (not a
; named node), so it is matched as a literal, not `(this)`.
;
; Note: this grammar does not model Groovy constructors at all —
; `Circle(double r) { ... }` (no return type, same name as the enclosing
; class) fails to parse as `function_definition`/`function_declaration`
; because both require a `type:` field; it instead misparses as a
; `function_call` immediately followed by a stray `closure`, with an ERROR
; node inside the argument list (confirmed via `normalize syntax ast`). So
; true "constructor delegation" (`this(...)` as the first statement of a
; constructor) is not reachable through a valid parse. This clause still
; matches the `this(args)` call *shape* wherever it legally appears (the
; token sequence `this ( args )` is otherwise ordinary Groovy syntax).
(function_call
  function: "this" @call)
