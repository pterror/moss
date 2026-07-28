; Complexity query for Haskell
; @complexity — nodes that increase cyclomatic complexity
; @nesting — nodes that increase nesting depth
;
; In Haskell's tree-sitter grammar, conditional, case, match, guard, and lambda
; are represented as expressions rather than statements.
;
; IMPORTANT: `match` is NOT a complexity node, despite the name suggesting
; pattern-matching. In this grammar every function equation body (even a
; trivial `f x = x + 1` with zero branches) is wrapped in its own `match`
; node (`function.match`), and every case/lambda_case/lambda_cases arm body
; is *also* wrapped in a `match` node (`alternative.match`). An earlier
; version of this file had `(match) @complexity`, which counted every single
; function's equation-body wrapper as a decision point — inflating the
; baseline complexity of every Haskell function by +1 minimum, and by one
; per arm for multi-arm case/lambda_case expressions even when those arms had
; no guards at all (confirmed via `normalize rank complexity`: a plain
; 4-arm, no-guard `case` reported complexity 7 instead of the correct 5).
; Verified via real parse (`normalize syntax query`) before removing.

; Complexity nodes
(conditional) @complexity
(case) @complexity
; `guard` is a grammar supertype alias, not a materializing node — its real
; subtypes are `boolean`/`let`/`pattern_guard` (confirmed via node-types.json
; and real parse: a captured guard's own node.kind() reports "boolean", never
; "guard"). tree-sitter's query engine resolves supertype names like this
; specially, so `(guard) @complexity` still matches every concrete subtype
; without needing to enumerate them — this is intentional, not an oversight.
(guard) @complexity
(lambda) @complexity

; multi_way_if / lambda_case / lambda_cases: GHC extensions (MultiWayIf,
; LambdaCase) in pervasive real-world use, structurally equivalent to
; case/conditional as branching constructs but previously entirely absent
; from this query — `\case` and `if | ... -> ...` contributed zero
; complexity no matter how many branches they had.
(multi_way_if) @complexity
(lambda_case) @complexity
(lambda_cases) @complexity

; Each case/lambda_case/lambda_cases arm is its own decision point (mirrors
; `match_arm` in rust.complexity.scm) — previously the arm bodies were only
; counted via the (now-removed) blanket `(match) @complexity`, which also
; double-counted every guarded arm's `guard` children.
(alternative) @complexity

; Nesting nodes
(conditional) @nesting
(case) @nesting
(match) @nesting
(multi_way_if) @nesting
(lambda_case) @nesting
(lambda_cases) @nesting
(function) @nesting
(class) @nesting
(instance) @nesting
