; Swift complexity query
; @complexity — nodes that increase cyclomatic complexity
; @nesting — nodes that increase nesting depth

; Complexity nodes
(if_statement) @complexity
(for_statement) @complexity
(while_statement) @complexity
(repeat_while_statement) @complexity
(switch_statement) @complexity
(catch_block) @complexity
(ternary_expression) @complexity
(nil_coalescing_expression) @complexity

; guard is Swift's early-exit branch construct (`guard cond else { return }`)
; — as much a branch as `if`, and an extremely common idiom. Verified via
; `normalize syntax query` that guard_statement is a real, distinct node type
; previously entirely absent from this file.
(guard_statement) @complexity

; Each switch case is its own branch (McCabe complexity counts one path per
; case), matching the convention already used for every other language with
; a match/switch construct in this codebase (Python's case_clause, Go's
; expression_case/type_case, Kotlin's when_entry, Java's switch_label). Swift
; previously only counted the switch_statement itself once, undercounting
; the complexity of every switch with more than one case — and Swift code
; leans on switch over enums heavily.
(switch_entry) @complexity

; Short-circuiting boolean operators (`&&`, `||`) are independent branch
; points, matching Kotlin's identical convention for its identically-named
; conjunction_expression/disjunction_expression node types (Swift's grammar
; uses the same node-type names for `&&`/`||`).
(conjunction_expression) @complexity
(disjunction_expression) @complexity

; Nesting nodes
(if_statement) @nesting
(for_statement) @nesting
(while_statement) @nesting
(repeat_while_statement) @nesting
(switch_statement) @nesting
(do_statement) @nesting
(function_declaration) @nesting
(class_declaration) @nesting
(lambda_literal) @nesting

; guard's `else` block is itself a nested statements block, same as if/while
; bodies above.
(guard_statement) @nesting
