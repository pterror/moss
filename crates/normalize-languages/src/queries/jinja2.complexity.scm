; Jinja2 complexity query
; @complexity — nodes that increase cyclomatic complexity
; @nesting     — nodes that increase nesting depth

; Complexity nodes
(for_statement) @complexity
(if_statement) @complexity
(elif_clause) @complexity

; Short-circuit boolean operators and the ternary/conditional expression add
; a decision point, matching the precedent set by python.complexity.scm
; ("and" @complexity, "or" @complexity, (conditional_expression) @complexity)
; for the equivalent Jinja2 constructs. Verified real grammar node types via
; `normalize syntax query -p <probe> "(and_expression) @c"` etc.
(and_expression) @complexity
(or_expression) @complexity
(ternary_expression) @complexity

; Nesting nodes
(for_statement) @nesting
(if_statement) @nesting
(macro_statement) @nesting
(call_statement) @nesting
