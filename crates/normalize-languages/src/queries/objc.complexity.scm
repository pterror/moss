; Complexity query for Objective-C
; @complexity — nodes that increase cyclomatic complexity
; @nesting — nodes that increase nesting depth

; Complexity nodes
(if_statement) @complexity
(switch_statement) @complexity
(while_statement) @complexity
(for_statement) @complexity

; do-while, case arms, short-circuit boolean operators, and the ternary
; operator all add a branch for cyclomatic-complexity purposes and were
; entirely missing here despite c.complexity.scm/cpp.complexity.scm already
; covering them for the C-family languages objc's grammar is built on
; (verified present via probe + node-types.json before adding).
(do_statement) @complexity
(case_statement) @complexity
(binary_expression operator: "&&" @complexity)
(binary_expression operator: "||" @complexity)
(conditional_expression) @complexity

; @try/@catch add a branch, mirroring cpp.complexity.scm's treatment of
; try_statement/catch_clause. throw_statement is intentionally NOT counted
; here (cpp.complexity.scm does count it, but a single throw site is a
; single exit, not a branch point — objc.cfg.scm already tracks it as
; @cfg.exit.throw; adding it to complexity would double-count relative to
; how this file treats return/break/continue, which are also not counted).
(try_statement) @complexity
(catch_clause) @complexity

; Nesting nodes
(if_statement) @nesting
(switch_statement) @nesting
(while_statement) @nesting
(for_statement) @nesting
(do_statement) @nesting
(try_statement) @nesting
(function_definition) @nesting
(class_interface) @nesting
(class_implementation) @nesting

; method_declaration (interface/protocol prototype, `- (void)foo;`) never
; has a body — it can't contain any complexity/nesting node, so nesting it
; (as the original query did) was a no-op. method_definition (the
; @implementation body, `- (void)foo { ... }`) is where all real control
; flow lives and was NOT nested at all — every if/while/switch inside every
; ObjC method implementation was being measured one nesting level too
; shallow. Verified via node-types.json (method_declaration and
; method_definition are distinct node types) and probe (method_definition
; has a `compound_statement` body child; method_declaration never does).
(method_definition) @nesting
