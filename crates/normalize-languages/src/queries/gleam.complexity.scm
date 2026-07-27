; Complexity query for Gleam
; @complexity — nodes that increase cyclomatic complexity
; @nesting — nodes that increase nesting depth
;
; Gleam is a functional language; complexity comes from case expressions
; and their branches, and if expressions.

; Complexity nodes
;
; Gleam has no `if` expression (branching is done via `case`), so there is no
; `if` node type in the grammar — confirmed via `normalize syntax query`
; (Query::new fails with "Invalid node type \"if\"").
(case) @complexity
(case_clause) @complexity

; Nesting nodes
(case) @nesting
(function) @nesting
(anonymous_function) @nesting
