; Ruby complexity query
; @complexity — nodes that increase cyclomatic complexity
; @nesting — nodes that increase nesting depth

; Complexity nodes
(if) @complexity
(unless) @complexity
(case) @complexity
(when) @complexity
(while) @complexity
(until) @complexity
(for) @complexity
(begin) @complexity
(rescue) @complexity
"and" @complexity
"or" @complexity
(conditional) @complexity

; Statement-modifier forms (`stmt if cond`, `stmt unless cond`, `stmt while
; cond`, `stmt until cond`) are distinct node types from their block forms
; (`if`/`unless`/`while`/`until`) in this grammar, not a different field
; layout of the same node — verified via `normalize syntax query`. Extremely
; common idiomatic Ruby; omitting them silently undercounts branch/loop
; complexity for any method that leans on statement modifiers instead of
; block syntax.
(if_modifier) @complexity
(unless_modifier) @complexity
(while_modifier) @complexity
(until_modifier) @complexity

; `rescue` as an inline expression modifier (`value = risky rescue default`)
; is a distinct `rescue_modifier` node, not a `rescue` clause inside a
; `begin`/`def` body. It's an implicit branch (try/fallback) and a very
; common idiom.
(rescue_modifier) @complexity

; Each `elsif` is its own node (nested under the outer `if`'s `alternative`
; field), not folded into a single `if` node the way some grammars do. Each
; elsif is an independent branch and must count separately, or an if/elsif/
; elsif/.../else chain is undercounted as a single branch point.
(elsif) @complexity

; Ruby 2.7+ pattern-matching `case ... in ...` parses as `case_match`/
; `in_clause`, entirely distinct node types from `case`/`when`. Without these,
; pattern-matching case statements (increasingly common in modern Ruby) are
; invisible to complexity counting.
(case_match) @complexity
(in_clause) @complexity

; Nesting nodes
(if) @nesting
(unless) @nesting
(case) @nesting
(case_match) @nesting
(while) @nesting
(until) @nesting
(for) @nesting
(begin) @nesting
(method) @nesting
(singleton_method) @nesting
(class) @nesting
(module) @nesting
(do_block) @nesting
(block) @nesting
