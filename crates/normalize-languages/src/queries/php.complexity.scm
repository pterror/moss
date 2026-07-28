; Complexity query for PHP
; @complexity — nodes that increase cyclomatic complexity
; @nesting — nodes that increase nesting depth

; Complexity nodes
(if_statement) @complexity
(for_statement) @complexity
(foreach_statement) @complexity
(while_statement) @complexity
(do_statement) @complexity
(case_statement) @complexity
(catch_clause) @complexity
(conditional_expression) @complexity

; PHP 8's `match` expression: each non-default arm is an independent
; branch, the same role `case_statement` plays for `switch`.
; `match_default_expression` (the `default => ...` arm) is intentionally
; excluded, mirroring `case_statement`'s own convention of not counting
; `default_statement` in `switch` — the default arm doesn't add a new
; decision path beyond what the other arms already account for. Verified
; the node shape via `normalize syntax ast` (`match_condition_list` nests
; one level deeper than `match_conditional_expression`'s siblings suggest —
; the same quirk the CFG remediation found for cfg.scm's match handling).
(match_conditional_expression) @complexity

; Short-circuit boolean operators, matching the `and`/`or` precedent already
; established for python.complexity.scm/ruby.complexity.scm: each one is an
; independent decision point. `binary_expression` is PHP's single generic
; binary-operator node (used for +, -, ==, ., instanceof, …), so these are
; matched via the `operator` field's literal token value, not the node kind,
; to avoid counting every other operator. `??` (null coalescing) is left
; uncounted — no precedent for it in this codebase's other languages'
; complexity queries (checked typescript.complexity.scm/
; javascript.complexity.scm), and PHP's short-circuit boolean set already
; has direct precedent to follow.
(binary_expression operator: "&&") @complexity
(binary_expression operator: "||") @complexity
(binary_expression operator: "and") @complexity
(binary_expression operator: "or") @complexity
(binary_expression operator: "xor") @complexity

; Nesting nodes
(if_statement) @nesting
(for_statement) @nesting
(foreach_statement) @nesting
(while_statement) @nesting
(do_statement) @nesting
(switch_statement) @nesting
(match_expression) @nesting
(try_statement) @nesting
(function_definition) @nesting
(method_declaration) @nesting
(class_declaration) @nesting
