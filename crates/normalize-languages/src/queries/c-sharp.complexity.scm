; C# complexity query
; @complexity — nodes that increase cyclomatic complexity
; @nesting — nodes that increase nesting depth

; Complexity nodes
(if_statement) @complexity
(for_statement) @complexity
(foreach_statement) @complexity
(while_statement) @complexity
(do_statement) @complexity
(switch_section) @complexity
(catch_clause) @complexity
(conditional_expression) @complexity
(binary_expression) @complexity

; Switch expression arms: `n switch { 1 => "a", 2 => "b", _ => "c" }`
; (C# 8+). A distinct node kind (`switch_expression`/`switch_expression_arm`,
; not `switch_statement`/`switch_section`) for the modern switch-expression
; form — entirely uncounted before this fix despite being a heavily used
; modern-C# idiom (the pre-existing sample.cs fixture's own `n switch {...}`
; block was never exercising this).
(switch_expression_arm) @complexity

; Nesting nodes
(if_statement) @nesting
(for_statement) @nesting
(foreach_statement) @nesting
(while_statement) @nesting
(do_statement) @nesting
(switch_statement) @nesting
(switch_expression) @nesting
(try_statement) @nesting
(method_declaration) @nesting
(class_declaration) @nesting
(lambda_expression) @nesting
