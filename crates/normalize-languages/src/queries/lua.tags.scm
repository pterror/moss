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
  name: (method_index_expression
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

; Note: `Foo:bar = function(...) end` is not valid Lua syntax — colon syntax
; is only valid for method definitions/calls, never as an assignment target
; (`method_index_expression` cannot appear inside `variable_list`). There is
; no assignment-form equivalent to capture here.
