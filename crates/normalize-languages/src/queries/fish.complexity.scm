; Complexity query for Fish shell
; @complexity — nodes that increase cyclomatic complexity
; @nesting — nodes that increase nesting depth

; Complexity nodes
(if_statement) @complexity
(else_if_clause) @complexity
(while_statement) @complexity
(for_statement) @complexity
(switch_statement) @complexity
(case_clause) @complexity

; `and`/`or` (fish's short-circuit chaining of the previous command's exit
; status, e.g. `test -f f; and echo yes; or echo no`) each wrap in a
; `conditional_execution` node and are each a real decision point, mirroring
; bash.complexity.scm's treatment of `&&`/`||` (see that file's comment for
; the reasoning on why only the logical-branch operators count). Verified via
; `normalize syntax ast` against a probe; this repo's own sample.fish already
; uses the idiom (`greet`'s `and`/`or` chain) and was previously uncounted.
(conditional_execution) @complexity

; Nesting nodes
(if_statement) @nesting
(while_statement) @nesting
(for_statement) @nesting
(switch_statement) @nesting
(function_definition) @nesting
