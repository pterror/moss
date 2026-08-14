; jq complexity query
; @complexity — nodes that increase cyclomatic complexity
; @nesting — nodes that increase nesting depth
;
; In jq, if/elif/reduce/foreach/and/or are anonymous keyword tokens, not
; wrapped in their own named node (the whole `if ... end`/`reduce ... )`
; construct is flattened directly into the enclosing `query` node — see
; jq.cfg.scm's header comment for the verified CST shape. Anonymous tokens
; are still queryable as literal strings (`"if"` etc, verified via
; `normalize syntax query`), so each decision point below is captured that
; way rather than skipped.

; Complexity nodes
(funcdef) @complexity
(elif) @complexity
(catch) @complexity
; Bare `if ... then ... end` (no `elif`) previously contributed *zero*
; complexity — only `(elif)` above was captured, so an if/else with no
; elif branch (the most common jq conditional form, e.g. every `if type ==
; "object" then ... else ... end` in this crate's own sample.jq) was
; silently undercounted relative to one with an elif. Each `if` is its own
; decision point, symmetric with `elif`.
("if") @complexity
; reduce/foreach are jq's iteration constructs (its only equivalent to a
; for/while loop in imperative languages); every other language's
; complexity.scm counts loop constructs. Verified via `normalize syntax
; query` against probe files — both are anonymous keyword tokens with no
; wrapping node, same shape as `if`.
("reduce") @complexity
("foreach") @complexity
; `and`/`or` short-circuit (the second operand is only evaluated when
; needed), matching the same "count only the actual short-circuit boolean
; operators" convention documented in bash.complexity.scm for `&&`/`||`.
("and") @complexity
("or") @complexity

; Nesting nodes
(funcdef) @nesting
(elif) @nesting
("if") @nesting
("reduce") @nesting
("foreach") @nesting
