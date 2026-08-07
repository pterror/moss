; Lua tags query
; Covers: function declarations, local function declarations, method assignments

; Global function declarations: function foo(...) ... end
(function_declaration
  name: (identifier) @name) @definition.function

; Namespaced function declarations: function Table.method(...) ... end
(function_declaration
  name: (dot_index_expression
    field: (identifier) @name)) @definition.method

; Method declarations: function Table:method(...) ... end
(function_declaration
  name:
    (method_index_expression
      method: (identifier) @name)) @definition.method

; Note: `local function foo(...) ... end` also parses as `function_declaration`
; with a leading `local` token (there is no distinct `local_function` node type
; in this grammar) — the global function pattern above already matches it.

; Method definitions via assignment: function Table:method(...) ... end
; or: Namespace.method = function(...) ... end
(assignment_statement
  (variable_list
    (dot_index_expression
      field: (identifier) @name))
  (expression_list
    (function_definition))) @definition.method

; Bare-name function-expression assignment: `local f = function(...) ... end`
; or `f = function(...) ... end`. `variable_list`'s `name` field (per
; arborium-lua's node-types.json, via the hidden `variable` supertype) allows
; bracket_index_expression, dot_index_expression, and identifier; the
; dot_index_expression form is handled above, this handles the identifier
; form. This is a very common Lua idiom (assigning an anonymous function
; expression to a local/global, as an alternative to `local function f()`)
; and was previously silently dropped. Matches javascript.tags.scm's
; convention of tagging `const f = function(){}`/`const f = () => {}` as
; @definition.function.
(assignment_statement
  (variable_list
    (identifier) @name)
  (expression_list
    (function_definition))) @definition.function

; Note: `Foo:bar = function(...) end` is not valid Lua syntax — colon syntax
; is only valid for method definitions/calls, never as an assignment target
; (`method_index_expression` cannot appear inside `variable_list`). There is
; no assignment-form equivalent to capture here.
;
; Note: `t[k] = function(...) end` (bracket_index_expression as the
; assignment target, e.g. a dynamic dispatch-table entry) is deliberately
; NOT captured — the key is a runtime-computed expression, not a static
; name, so there is nothing meaningful to put in @name. Matches
; go.tags.scm's identical exclusion of computed collection elements.

; ---------------------------------------------------------------------------
; Call references
; ---------------------------------------------------------------------------
; Mirrors lua.calls.scm's callee-variant coverage (see that file's header
; comment) for the subset that has a static name — this was ported late
; (lua.calls.scm had it, lua.tags.scm didn't), the same class of gap
; documented as bug #5 in docs/query-testing-methodology.md's Rust example.
(function_call
  name: [
    (identifier) @name
    (dot_index_expression field: (identifier) @name)
    (method_index_expression method: (identifier) @name)
    ; Bracket-dispatched call: handlers["key"](), TABLE[i]() — no statically
    ; resolvable callee name; report the subscripted container's name as a
    ; best-effort approximation, matching python.tags.scm's identical
    ; convention for `handlers["key"]()`.
    (bracket_index_expression
      table: [
        (identifier) @name
        (dot_index_expression field: (identifier) @name)
      ])
  ]) @reference.call
