; Complexity query for ReScript (ML-like JS language)
; @complexity — nodes that increase cyclomatic complexity
; @nesting — nodes that increase nesting depth
;
; ReScript complexity comes from if expressions, switch expressions,
; individual match arms, loops, and exception handling.
;
; `for_expression`/`while_expression`/`try_expression` were entirely
; unhandled — verified present in arborium-rescript-2.17.0's node-types.json
; and via `normalize syntax ast`: ReScript has real imperative loops (for/
; while, used for Belt.Array/mutable-ref iteration) and try/catch (whose
; catch arms parse as `switch_match`, already covered below), none of
; which contributed to the complexity/nesting score before this fix.

; Complexity nodes
(if_expression) @complexity
(switch_expression) @complexity
(switch_match) @complexity
(ternary_expression) @complexity
(for_expression) @complexity
(while_expression) @complexity
(try_expression) @complexity

; Nesting nodes
(if_expression) @nesting
(switch_expression) @nesting
(function) @nesting
(block) @nesting
(for_expression) @nesting
(while_expression) @nesting
(try_expression) @nesting
