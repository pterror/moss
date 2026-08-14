; Complexity query for Perl
; @complexity — nodes that increase cyclomatic complexity
; @nesting — nodes that increase nesting depth

; Complexity nodes
;
; Perl has two structurally distinct for-loop node types (same distinction
; documented in perl.cfg.scm): for_statement (foreach form) and
; cstyle_for_statement (C-style `for (init; cond; step) {}`) — verified via
; `normalize syntax query` against a C-style-for probe, which parses as
; cstyle_for_statement, a type for_statement's pattern never matches.
(conditional_statement) @complexity
(loop_statement) @complexity
(for_statement) @complexity
(cstyle_for_statement) @complexity
(conditional_expression) @complexity

; Nesting nodes
(conditional_statement) @nesting
(loop_statement) @nesting
(for_statement) @nesting
(cstyle_for_statement) @nesting
(subroutine_declaration_statement) @nesting
(package_statement) @nesting
