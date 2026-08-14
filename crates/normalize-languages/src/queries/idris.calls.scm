; Idris calls query
; @call — function application nodes
; @call.qualifier — namespace qualifier for qualified calls
;
; Idris uses juxtaposition for function application, like Haskell.
; The grammar represents expressions as `exp_name` nodes for named references
; (both functions being applied and plain references). There is no distinct
; application node — `function` declarations contain `rhs` with expressions.
; Capture `exp_name` children that are simple names (loname = lowercase, caname = uppercase).

; Simple name reference / call: foo x y
(exp_name
  (loname) @call)

; Qualified reference / call: Module.foo x y
(exp_name
  (qualified_loname) @call)

; Constructor call: Foo x
(exp_name
  (caname) @call)

; Qualified constructor: Module.Foo x
(exp_name
  (qualified_caname) @call)

; Operator used as a value / passed as a function argument: foldr (+) 0 xs
; `exp_name` also allows `operator`/`qualified_operator` children (confirmed
; via `normalize syntax query` against a probe file — `foldr (+) 0 xs`
; produces `(exp_name (operator))`) and `Prelude.(+) 1 2` produces
; `(exp_name (qualified_operator))`. Passing an operator as a first-class
; function value is a pervasive functional-programming idiom (foldr/foldl
; with `(+)`/`(::)`/etc.), not a rare edge case.
(exp_name
  (operator) @call)

(exp_name
  (qualified_operator) @call)

; NOTE: `exp_name` also allows `dot_operator`/`qualified_dot_operators`
; children per node-types.json, but no real-world or synthetic probe source
; was found that actually produces either shape nested inside `exp_name`
; (the bare `.` composition operator parses as a plain `operator` token
; directly, not wrapped in `exp_name`/`dot_operator`) — left unhandled per
; "verify against real parse output, not node-types.json alone."
