; Complexity query for F#
; @complexity — nodes that increase cyclomatic complexity
; @nesting — nodes that increase nesting depth
;
; F# is a functional-first language; complexity comes from if/elif expressions,
; match rules, loops, try blocks, and boolean infix operators (&&, ||).
;
; `infix_expression` covers EVERY binary operator — arithmetic (+, -, *, /),
; comparison (=, <, >), string concat (^), pipelines (|>), as well as the
; short-circuit boolean operators (&&, ||). A blanket `(infix_expression)
; @complexity` counted all of them: verified via `normalize syntax query`,
; a two-line probe with only `+`/`-`/`*` (no branches, no booleans at all)
; produced 3 @complexity hits, and the 99-line sample.fs fixture (which has
; no `&&`/`||` at all) produced 25 — every arithmetic expression in the
; file. `infix_op` has no field name (grammar declares `"fields": {}` on
; `infix_expression`), so the operator itself is matched positionally and
; filtered by its token text down to just `&&`/`||`.

; Complexity nodes
(if_expression) @complexity
(rule) @complexity
(for_expression) @complexity
(while_expression) @complexity
(try_expression) @complexity
(infix_expression
  (infix_op) @_op
  (#match? @_op "^(&&|\\|\\|)$")
) @complexity

; Nesting nodes
(if_expression) @nesting
(for_expression) @nesting
(while_expression) @nesting
(try_expression) @nesting
(function_or_value_defn) @nesting
(member_defn) @nesting
(module_defn) @nesting
(type_definition) @nesting
