; Dockerfile tags query
; @name            — the symbol name
; @definition.*    — the definition node

; Build stage names (FROM ... AS name)
;
; `as:` is a field on `from_instruction` (see imports.scm's comment — there is
; no intermediate `as_instruction` node in this grammar).
(from_instruction
  as: (image_alias) @name) @definition.module

; ARG declarations
;
; `arg_instruction` has two fields — `name` (always `unquoted_string`) and
; `default` (`unquoted_string` | `double_quoted_string` | `single_quoted_string`).
; Both can be `unquoted_string` (`ARG VERSION=1.0`), so an unconstrained
; `(unquoted_string) @name` matches the default value too, producing a
; spurious @definition.constant for "1.0". Field-anchor to `name:` only.
(arg_instruction
  name: (unquoted_string) @name) @definition.constant

; ENV declarations
;
; Same issue as ARG: `env_pair`'s `value` field also allows `unquoted_string`
; (`ENV KEY value` / `ENV KEY=value`), so an unconstrained pattern doubled up
; on the value text as a spurious symbol. Field-anchor to `name:` only —
; `env_pair.name` is exclusively `unquoted_string` per node-types.json.
(env_instruction
  (env_pair
    name: (unquoted_string) @name)) @definition.constant
