; SCSS complexity query
; @complexity — nodes that increase cyclomatic complexity
; @nesting — nodes that increase nesting depth

; Complexity nodes
(if_statement) @complexity
; `@else if ...` is a distinct `else_if_clause` node (not a nested
; `if_statement` inside `else_clause`) — confirmed via `normalize syntax
; ast`: `if_statement`'s children are `block` / `else_clause` /
; `else_if_clause` directly, each `else_if_clause` sibling adding its own
; branch. Each one is its own decision point and must add complexity.
(else_if_clause) @complexity
(for_statement) @complexity
(each_statement) @complexity
(while_statement) @complexity

; Nesting nodes
(if_statement) @nesting
(else_if_clause) @nesting
(for_statement) @nesting
(each_statement) @nesting
(while_statement) @nesting
