; Fish shell calls query
; @call — command being executed
; @call.qualifier — not applicable
;
; Fish represents command invocations as `command` nodes with a `name` field.
; `command.name` allows 12 node-type variants in node-types.json (arborium-fish
; 2.17.0), not just `word` — verified via `normalize syntax query` against real
; probes: `$cmd arg` gives `variable_expansion` (dispatch-by-variable, e.g.
; `$EDITOR file`), `"$prefix"ho` gives `concatenation`, `(echo sub) arg` gives
; `command_substitution` (dynamic command name from a subshell's stdout), and
; quoted command names (`"echo" arg`, `'echo' arg`) give
; `double_quote_string`/`single_quote_string`. A `(word)`-only constraint
; silently dropped all of these. Use the wildcard to cover every variant,
; mirroring the `argument:` field's existing wildcard treatment below.

; Command invocation: some_command arg1 arg2
(command
  name: (_) @call)
