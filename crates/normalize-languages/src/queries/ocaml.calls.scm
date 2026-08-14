; OCaml calls query
; @call — call expression nodes
; @call.qualifier — qualifier/receiver for module-qualified calls
;
; OCaml uses juxtaposition for function application: `f x y` is represented
; as a single `application_expression` with multiple `argument` children
; (verified via `normalize syntax ast` — `application_expression.argument`
; is `multiple=true` in arborium-ocaml 2.17.0's node-types.json), not
; nested `application_expression`s.
;
; `application_expression.function` is typed as `_simple_expression`, a
; 25-variant supertype. Only 3 of those 25 are meaningful call-site shapes
; that carry a static callee name (`value_path`, `method_invocation`,
; `field_get_expression`); the rest (parenthesized/computed callees) get a
; best-effort whole-node capture, matching the convention already used by
; javascript.calls.scm / lua.calls.scm for the same class of dynamic callee.
;
; `value_path`'s children are `module_path`, `parenthesized_operator`, or
; `value_name` (per node-types.json) — the unqualified-name pattern below
; must anchor on `.` (first child) or it silently double-matches every
; module-qualified call too (`Module.func arg` produced two @call captures
; before this fix: one from this pattern matching the trailing `value_name`
; child unanchored, one from the module-qualified pattern below — verified
; via `normalize syntax query` against `Module.func arg`). Module-qualified
; calls (`List.map`, `String.concat`, `Printf.printf`, …) are one of the
; single most common call shapes in real OCaml, so this silently inflated
; call counts for nearly every stdlib/qualified call site.

; Function application: f x
(application_expression
  function: (value_path
    . (value_name) @call))

; Module-qualified call: Module.func arg
(application_expression
  function: (value_path
    (module_path) @call.qualifier
    (value_name) @call))

; Operator-as-function call: ( + ) 1 2
(application_expression
  function: (value_path
    . (parenthesized_operator) @call))

; Module-qualified operator call: M.( + ) 1 2
(application_expression
  function: (value_path
    (module_path) @call.qualifier
    (parenthesized_operator) @call))

; Method call: obj#method arg — but also obj#method with NO arguments
; (`method_invocation` is itself a complete, self-invoking call regardless
; of whether it is further applied to arguments; it does not need to be
; wrapped in an `application_expression` to represent a call, unlike plain
; function application). This must stay a single bare top-level pattern,
; not one nested under `application_expression`: `method_invocation` is the
; same node either way (`application_expression` just wraps it when extra
; arguments follow), so a second application_expression-scoped pattern
; would double-match every argument-taking method call (verified via
; `normalize syntax query`, same failure mode as the value_path anchoring
; bug documented above).
(method_invocation
  (_) @call.qualifier
  (method_name) @call)

; Field-function call: record.field arg — a record field holding a function
; value, applied directly (common with first-class-function record fields,
; e.g. `{ run : unit -> int }`).
(application_expression
  function: (field_get_expression
    record: (_) @call.qualifier
    field: (field_path
      (field_name) @call)))

; Parenthesized-expression callee: (if c then f else g) x — no static name;
; best-effort whole-node capture, matching javascript.calls.scm/
; lua.calls.scm's identical IIFE-style case.
(application_expression
  function: (parenthesized_expression) @call)
