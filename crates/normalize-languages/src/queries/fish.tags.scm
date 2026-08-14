; Fish shell tags query
;
; `function_definition.name` allows the same 12 node-type variants as
; `command.name` (arborium-fish 2.17.0 node-types.json) — verified via
; `normalize syntax query`: `function "quoted name" ... end` is real, working
; fish syntax (quoting lets a function name contain spaces/special chars) and
; parses its name as `double_quote_string`, not `word`. A `(word)`-only
; constraint silently dropped it (0 matches against a probe with 2 function
; definitions, one quoted). Use the wildcard to cover every variant.

(function_definition
  name: (_) @name) @definition.function
