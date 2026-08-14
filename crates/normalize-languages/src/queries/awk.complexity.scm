; Awk complexity query
; @complexity — nodes that increase cyclomatic complexity
; @nesting — nodes that increase nesting depth

; Complexity nodes
(if_statement) @complexity
(while_statement) @complexity
(for_statement) @complexity
(for_in_statement) @complexity
(ternary_exp) @complexity

; switch/case (gawk extension): each case is a decision point, mirroring
; javascript.complexity.scm's `(switch_case) @complexity` convention
; (`switch_default` is not a decision branch — no condition — and is not
; counted, same as javascript). Verified via `normalize syntax ast` that
; `switch_statement`/`switch_case` are real, produced node types, not just
; declared-but-unused entries in node-types.json.
(switch_case) @complexity

; Nesting nodes
(if_statement) @nesting
(while_statement) @nesting
(for_statement) @nesting
(for_in_statement) @nesting
(switch_statement) @nesting
