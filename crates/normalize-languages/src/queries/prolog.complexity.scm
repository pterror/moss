; Prolog complexity query
; @complexity — nodes that increase cyclomatic complexity
; @nesting — nodes that increase nesting depth

; Complexity nodes (each clause adds complexity via pattern matching)
(clause_term) @complexity

; Disjunction (;) and if-then (->) inside a clause body are additional
; decision points beyond clause count — a predicate with an if-then-else
; body has strictly more branches than one without, but clause_term alone
; can't see that. Mirrors erlang.complexity.scm's (if_clause)/(cr_clause)
; treatment and the same operator-text matching prolog.cfg.scm already
; uses for this grammar (no dedicated if_then/if_then_else node type
; exists — see prolog.cfg.scm's header comment).
(operator_notation
  operator: (semicolon)) @complexity

(operator_notation
  operator: (binary_operator) @_op
  (#eq? @_op "->")) @complexity

; Nesting nodes
(clause_term) @nesting

(operator_notation
  operator: (semicolon)) @nesting

(operator_notation
  operator: (binary_operator) @_op
  (#eq? @_op "->")) @nesting
