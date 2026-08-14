; Complexity query for Julia
; @complexity — nodes that increase cyclomatic complexity
; @nesting — nodes that increase nesting depth

; Complexity nodes
(if_statement) @complexity
(elseif_clause) @complexity
(for_statement) @complexity
(while_statement) @complexity
(ternary_expression) @complexity
(catch_clause) @complexity

; Short-circuit boolean operators add a branch, matching the convention in
; python.complexity.scm ("and"/"or" @complexity). Julia represents `&&`/`||`
; as a plain binary_expression with an `operator` child carrying the token
; text — verified via `normalize syntax ast`/`syntax query` against a probe
; file (`a && b || c` parses as nested binary_expression nodes). This
; codebase's predicate evaluator (normalize-languages::query_predicates)
; only supports match?/not-match?/eq?/not-eq? — no any-of? — so the two
; operators are matched with a single alternation regex, not #any-of?.
(binary_expression
  (operator) @_op
  (#match? @_op "^(&&|\\|\\|)$")) @complexity

; Comprehensions/generators add a branch per the implicit iteration+filter,
; matching python.complexity.scm's list/dict/set/generator comprehension
; treatment. `comprehension_expression` is the bracketed form (`[x for x in
; ...]`), `generator` is the bare parenthesized form used as a call argument
; (`sum(x for x in ...)`) — both confirmed distinct node types via
; node-types.json and verified to parse as expected via `normalize syntax
; ast`.
(comprehension_expression) @complexity
(generator) @complexity

; Nesting nodes
(if_statement) @nesting
(for_statement) @nesting
(while_statement) @nesting
(try_statement) @nesting
(function_definition) @nesting
(macro_definition) @nesting
(module_definition) @nesting
(struct_definition) @nesting
