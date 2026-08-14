; Complexity query for PowerShell
; @complexity — nodes that increase cyclomatic complexity
; @nesting — nodes that increase nesting depth

; Complexity nodes
(if_statement) @complexity
(elseif_clause) @complexity
(while_statement) @complexity
(for_statement) @complexity
(foreach_statement) @complexity
(do_statement) @complexity
(catch_clause) @complexity
(trap_statement) @complexity

; switch_statement itself is intentionally not counted as a unit: each
; concrete switch_clause below already contributes one unit per branch,
; mirroring the convention `case_statement`/`match_arm` use elsewhere
; (bash.complexity.scm, rust.complexity.scm, php.complexity.scm, etc.) of
; counting per-arm rather than per-container.
;
; The `default` clause parses as a `switch_clause` whose
; `switch_clause_condition` is empty (no named child) in the plain form —
; but under `switch -Regex`/`switch -Wildcard`, EVERY clause's condition
; (including non-default pattern clauses) has an empty
; switch_clause_condition too (verified via `normalize syntax ast`: the
; regex/wildcard pattern text is not wrapped in a named child node at all
; in that grammar path, a real, verified grammar quirk, not a guess). So
; "has a named child" cannot distinguish default from non-default under
; -Regex/-Wildcard. Matching on the condition's own text instead
; (`#not-eq? @cond "default"`) works uniformly across all three switch
; forms, since a literal pattern/value can never itself render as the bare
; unquoted token `default`.
(switch_clause
  (switch_clause_condition) @_cond (#not-eq? @_cond "default")) @complexity

; Nesting nodes
(if_statement) @nesting
(while_statement) @nesting
(for_statement) @nesting
(foreach_statement) @nesting
(do_statement) @nesting
(switch_statement) @nesting
(try_statement) @nesting
(function_statement) @nesting
(class_statement) @nesting
