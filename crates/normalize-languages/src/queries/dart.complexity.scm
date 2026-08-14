; Dart complexity query
; @complexity — nodes that increase cyclomatic complexity
; @nesting — nodes that increase nesting depth

; Complexity nodes
(if_statement) @complexity
(for_statement) @complexity
(while_statement) @complexity
(do_statement) @complexity
(switch_statement_case) @complexity
; Dart 3 pattern-matching switch expression: `switch (n) { 0 => ..., _ => ... }`.
; switch_expression is a distinct node from switch_statement (it's an
; expression, not a statement) with its own switch_expression_case children
; — entirely unmatched before, silently dropping every switch-expression arm
; from complexity counting. Verified via `normalize syntax query`.
(switch_expression_case) @complexity
(catch_clause) @complexity
(conditional_expression) @complexity
(logical_and_expression) @complexity
(logical_or_expression) @complexity
; Null-coalescing `??` is a short-circuiting branch point, same as `&&`/`||`
; above; if_null_expression was the only other short-circuit binary operator
; in the grammar and was the only one not counted.
(if_null_expression) @complexity

; Nesting nodes
(if_statement) @nesting
(for_statement) @nesting
(while_statement) @nesting
(do_statement) @nesting
(switch_statement) @nesting
(switch_expression) @nesting
(try_statement) @nesting
(function_body) @nesting
(class_definition) @nesting
(function_expression) @nesting
