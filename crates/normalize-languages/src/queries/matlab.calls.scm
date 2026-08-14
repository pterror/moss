; MATLAB calls query
; @call — function call expression
; @call.qualifier — not applicable (MATLAB method calls go through field_expression,
;   which nests a `function_call` in its `field:` position — already matched below
;   without needing a separate qualifier pattern)
;
; MATLAB function calls are represented as `function_call` nodes with a `name`
; field containing an `identifier`. Command-syntax calls (`disp x`) appear as
; `command` nodes with a `command_name` child.
;
; Verified against arborium-matlab 2.17.0 node-types.json: `function_call`'s
; `name` field allows three node types, not just `identifier`:
;   ["function_call", "identifier", "indirect_access"]

; Standard function call: func(args...)
(function_call
  name: (identifier) @call)

; Dynamic-field call: s.(fieldExpr)(args...) — the callee is a computed field
; name in parens. Only the plain-identifier form of the computed expression
; yields a usable static call name (`s.(handlerName)(...)`); when the
; parenthesized expression is itself an arbitrary computed sub-expression
; (binary/function-call/etc — `indirect_access`'s grammar allows those too),
; there is genuinely no static name to extract, so it is intentionally left
; uncaptured rather than fabricated.
(function_call
  name: (indirect_access
    (identifier) @call))

; NOT HANDLED (documented, not fabricated): `function_call name: (function_call)`
; — a chained/curried call whose callee is itself the result of another call,
; e.g. `getFunc()(3)`. The grammar allows this (confirmed via node-types.json
; and a real parse), but there is no static identifier to name as the callee
; — capturing the whole inner `function_call` node as `@call` would report a
; nonsensical "call name" (the source text of the entire inner call
; expression). This idiom is also rare in idiomatic MATLAB, which almost
; always binds an intermediate function handle to a variable first
; (`f = getFunc(); f(3);`) rather than chaining call syntax directly.

; Command syntax: command arg (e.g. `disp x`, `clear all`)
; `import` is excluded: it uses the same command-syntax parse (grammar has no
; dedicated import-statement node — see matlab.imports.scm) but is a language
; statement, not a function call; without this exclusion every `import pkg.X`
; line was double-counted as a spurious call to something named "import"
; (kind `command_name`, not a real callable).
(command
  (command_name) @call
  (#not-eq? @call "import"))
