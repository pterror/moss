; Meson complexity query
; @complexity — nodes that increase cyclomatic complexity
; @nesting — nodes that increase nesting depth

; Complexity nodes
(if_command) @complexity
(foreach_command) @complexity
(if_condition) @complexity

; `elseif_command` is a distinct node type from `if_command`/`if_condition`
; (verified against arborium meson node-types.json + a real if/elif/elif/else
; probe via `normalize syntax query`) and was previously uncaptured: an
; if/elif/elif/else chain produced the same complexity count (2) as a bare
; `if`, silently undercounting every additional branch. Each `elif` is its
; own decision point, so it counts toward complexity like `if_command` does;
; `else_command` is not a decision point and correctly stays uncounted.
(elseif_command) @complexity

; Nesting nodes
; elseif/else are siblings of if at the same nesting depth (not an
; additional nested level), so only the top-level if_command contributes
; nesting depth, matching common cyclomatic-complexity practice.
(if_command) @nesting
(foreach_command) @nesting
