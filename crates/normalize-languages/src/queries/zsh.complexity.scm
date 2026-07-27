; Complexity query for Zsh
;
; IMPORTANT — UNVERIFIED AT RUNTIME, KNOWN-BROKEN GRAMMAR:
; This query is schema-correct (checked against arborium-zsh's node-types.json)
; but its runtime capture behavior is unverified: the vendored arborium-zsh
; grammar ships "externals": [] and no scanner.c (upstream tree-sitter-zsh has
; a ~92KB hand-written one), so real zsh source parses into ERROR-laden trees
; rather than proper if/for/while/case nodes. This is a packaging defect in
; bearcove/arborium, not fixable by a query rewrite here. See zsh.cfg.scm's
; header and TODO.md for the full investigation, and
; `normalize_languages::known_broken_grammar("zsh")` for the runtime-facing
; surface of this fact.
;
; @complexity — nodes that increase cyclomatic complexity
; @nesting — nodes that increase nesting depth
;
; Zsh is very similar to bash; complexity comes from if/elif, for, while,
; case, and pipelines.

; Complexity nodes
(if_statement) @complexity
(elif_clause) @complexity
(for_statement) @complexity
(while_statement) @complexity
(case_statement) @complexity
(case_item) @complexity
(pipeline) @complexity

; Nesting nodes
(if_statement) @nesting
(for_statement) @nesting
(while_statement) @nesting
(case_statement) @nesting
(function_definition) @nesting
(subshell) @nesting
