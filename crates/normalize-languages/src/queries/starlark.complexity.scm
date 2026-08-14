; Complexity query for Starlark (Bazel build language)
; @complexity — nodes that increase cyclomatic complexity
; @nesting — nodes that increase nesting depth
;
; Starlark is Python-like; complexity comes from if statements, for loops,
; and inline conditional (ternary) expressions.

; Complexity nodes
(if_statement) @complexity
(elif_clause) @complexity
(for_statement) @complexity
(conditional_expression) @complexity
"and" @complexity
"or" @complexity
; List/dict comprehensions carry an implicit for + optional if (per
; arborium-starlark's node-types.json: `for_in_clause` and `if_clause`
; children), the same implicit control flow python.complexity.scm already
; counts for Python's equivalent comprehension nodes. Verified via
; `normalize syntax ast` that Starlark's grammar has no separate
; set_comprehension/generator_expression node types (Starlark has neither
; sets nor generators), so only these two apply.
(list_comprehension) @complexity
(dictionary_comprehension) @complexity

; Nesting nodes
(if_statement) @nesting
(for_statement) @nesting
(function_definition) @nesting
