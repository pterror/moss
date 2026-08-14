; Complexity query for Nix
; @complexity — nodes that increase cyclomatic complexity
; @nesting — nodes that increase nesting depth
;
; Nix is a purely functional language; complexity comes from if-then-else
; expressions, assert expressions, and short-circuiting boolean operators.

; Complexity nodes
(if_expression) @complexity
(assert_expression) @complexity

; Short-circuiting boolean operators (&&, ||) are independent branch points,
; same convention as bash.complexity.scm, c.complexity.scm, cpp.complexity.scm,
; php.complexity.scm, elixir.complexity.scm, and dart.complexity.scm's `??`.
; binary_expression.operator (per arborium-nix's node-types.json) allows:
; !=, &&, *, +, ++, -, ->, /, //, <, <=, ==, >, >=, ||. Only && and || are
; short-circuiting; verified via `normalize syntax query` against a probe
; file with `x != null && y > 0 || z`.
(binary_expression operator: "&&") @complexity
(binary_expression operator: "||") @complexity

; Nesting nodes
(if_expression) @nesting
(with_expression) @nesting
(let_expression) @nesting
(function_expression) @nesting
