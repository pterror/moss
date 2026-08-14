; OCaml complexity query
; @complexity — nodes that increase cyclomatic complexity
; @nesting — nodes that increase nesting depth

; Complexity nodes
(if_expression) @complexity
(match_expression) @complexity
(match_case) @complexity
; for/while loops and try/with exception handling were previously missing
; despite if/match already being covered — mirrors rust.complexity.scm
; (for_expression/while_expression) and python.complexity.scm/
; java.complexity.scm (try_statement/catch_clause) treating loops and
; exception-handling branches as complexity contributors. try's `with`
; arms reuse `match_case`, already counted above.
(for_expression) @complexity
(while_expression) @complexity
(try_expression) @complexity

; Nesting nodes
(let_expression) @nesting
(module_definition) @nesting
(match_expression) @nesting
(if_expression) @nesting
(for_expression) @nesting
(while_expression) @nesting
(try_expression) @nesting
