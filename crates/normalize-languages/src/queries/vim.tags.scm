; Vim tags query
; Covers: function definitions and augroup definitions

; Function definitions: function! FunctionName(args) ... endfunction
; The function_definition node contains a function_declaration with a name field.
(function_definition
  (function_declaration
    name: (identifier) @name)) @definition.function

; Scoped function: function! s:Foo(), g:Bar() — scoped_identifier.
; (The autoload-namespaced form, function! foo#bar#Baz(), is NOT a
; scoped_identifier — the `#` separators are part of a plain `identifier`
; token, verified via `normalize syntax query`; it's already covered by
; the plain-identifier pattern above.)
(function_definition
  (function_declaration
    name: (scoped_identifier) @name)) @definition.function

; Dict-method function: function! s:obj.Method() ... endfunction — the
; `name` field allows `field_expression` in addition to `identifier` and
; `scoped_identifier` (verified via `normalize syntax query` against a
; `function! g:Dict.Other()` probe: the name field's node kind is
; field_expression, text "g:Dict.Other"). This is the standard vim OO
; pattern for attaching a function to a dictionary as a bound method.
(function_definition
  (function_declaration
    name: (field_expression) @name)) @definition.function

; Augroup definitions: augroup MyGroup ... augroup END
; The augroup_statement node contains an augroup_name child.
(augroup_statement
  (augroup_name) @name) @definition.module
