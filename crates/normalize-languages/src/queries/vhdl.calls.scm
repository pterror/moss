; VHDL calls query
; @call — function/procedure call nodes
; @call.qualifier — qualifier for selected (package-qualified) calls
;
; VHDL has two call forms:
;   - ambiguous_name: used in expressions (parser can't distinguish function
;     calls from array indexing without type info, so uses ambiguous_name)
;   - function_call: used when parser can resolve it unambiguously
;   - procedure_call_statement: used as statements, has a `procedure` field
;
; Callee-name variants (verified via `normalize syntax query` against real
; parse output, not node-types.json alone — see
; crates/normalize-languages/tests/query_fixtures.rs vhdl_calls_completeness):
;   - simple_name: plain identifier call, e.g. `foo(x)`
;   - extended_simple_name: extended-identifier call, e.g. `\My Func\(x)`
;     (reachable via ambiguous_name when unqualified/positional args, and
;     via function_call when the parser can disambiguate, e.g. named
;     association `\My Func\(a => 1)`)
;   - operator_symbol: operator-function call, e.g. `"+"(a, b)` — always
;     parses as the unambiguous function_call form since an operator symbol
;     can never be an array name
; function_call.function also allows `attribute_name` per node-types.json,
; but no real construct producing that shape was found — attribute-style
; calls (`type'image(x)`) parse as a standalone `attribute_name` node, not
; nested under function_call. See the dedicated attribute-call pattern below.

; Function call in expression context (ambiguous with array indexing)
(ambiguous_name
  prefix: (simple_name) @call)

; Function call to an extended-identifier name, e.g. `\My Func\(x)`
(ambiguous_name
  prefix: (extended_simple_name) @call)

; Package-qualified ambiguous call: pkg.func(args) / pkg.\My Func\(args)
(ambiguous_name
  prefix: (selected_name
    prefix: (_) @call.qualifier
    suffix: [(simple_name) (extended_simple_name) (operator_symbol)] @call))

; Explicit function_call form (unambiguous)
(function_call
  function: [(simple_name) (extended_simple_name) (operator_symbol)] @call)

; Package-qualified explicit function call: pkg.func(args) / pkg."+"(a, b)
(function_call
  function: (selected_name
    prefix: (_) @call.qualifier
    suffix: [(simple_name) (extended_simple_name) (operator_symbol)] @call))

; Attribute-function call, e.g. `integer'image(x)`, `t'val(n)` — VHDL's
; predefined attribute-functions. `attribute_name` also covers plain
; attribute references with no call semantics (`clk'event`, `sig'stable`),
; which have no `(expression)` child; only capture the call form.
; Verified: only `predefined_designator` can co-occur with an `(expression)`
; child — `simple_name`/`extended_simple_name` designators (user-defined
; attributes) are never reachable with an expression argument in this
; grammar (`normalize syntax query` reports "Impossible pattern" for both),
; matching the VHDL semantics that only predefined attributes take
; function-call-style parameters.
(attribute_name
  designator: (predefined_designator) @call
  (expression))

; Procedure call: some_proc(args); / \My Proc\(args);
(procedure_call_statement
  procedure: [(simple_name) (extended_simple_name)] @call)

; Package-qualified procedure call: pkg.proc(args); / pkg.\My Proc\(args);
(procedure_call_statement
  procedure: (selected_name
    prefix: (_) @call.qualifier
    suffix: [(simple_name) (extended_simple_name)] @call))
