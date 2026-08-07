; Bash complexity query
; @complexity — nodes that increase cyclomatic complexity
; @nesting — nodes that increase nesting depth

; Complexity nodes
(if_statement) @complexity
(elif_clause) @complexity
(for_statement) @complexity
(c_style_for_statement) @complexity
(while_statement) @complexity
(case_statement) @complexity
(case_item) @complexity
(pipeline) @complexity
(list) @complexity
(ternary_expression) @complexity

; `binary_expression` is one overloaded node type covering every bash binary
; operator: arithmetic (`+`, `-`, `*=`, ...), comparison (`<`, `==`, ...),
; AND the `&&`/`||` short-circuit operators — unlike C-like grammars where
; assignment/arithmetic typically get their own node kind. An unconstrained
; `(binary_expression) @complexity` (the pattern used by c-sharp/java/go)
; would count every `(( i < count ))` loop condition and every
; `(( total += num ))` arithmetic update as an extra decision point, wildly
; inflating complexity for ordinary arithmetic-heavy shell scripts. Only the
; two logical short-circuit operators represent an actual decision point, so
; each is constrained by its `operator:` field text.
(binary_expression operator: "&&") @complexity
(binary_expression operator: "||") @complexity
; `-a`/`-o` (POSIX `test`/`[ ]` logical AND/OR, deprecated by bash's own
; manual in favor of `&&`/`||` between separate `[ ]` invocations) produce no
; hits anywhere in this repo's own shell scripts — not added, to avoid an
; unverified clause; add if a real occurrence surfaces.

; Nesting nodes
(if_statement) @nesting
(for_statement) @nesting
(c_style_for_statement) @nesting
(while_statement) @nesting
(case_statement) @nesting
(function_definition) @nesting
(subshell) @nesting
