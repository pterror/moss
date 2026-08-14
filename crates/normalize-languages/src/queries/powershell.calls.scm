; PowerShell calls query
; @call — command invocation nodes
; @call.qualifier — not applicable
;
; PowerShell represents command calls as `command` nodes with a
; `command_name` field. That field allows two node-type variants
; (verified against node-types.json's `command.fields.command_name.types`
; and against real parse output via `normalize syntax ast`):
;   - `command_name`      — a static/literal command name: Get-Item
;   - `command_name_expr` — a dynamic invocation target: & $cmdName,
;     . $scriptPath, . "path/to/script.ps1" (dot-sourcing also parses as
;     a `command` with this field shape, and is legitimately a call as
;     well as an import — dot-sourcing both runs and imports the script)
; Both must be matched or every `&`/`.`-invoked dynamic call is silently
; dropped from extraction.
(command
  command_name: (command_name) @call)

(command
  command_name: (command_name_expr) @call)

; Method calls ($obj.Method()) and static calls ([Type]::Method()) parse
; as `invokation_expression`, not `command` — this is a different call
; shape entirely (not the "& or . operator" invocation the old comment
; here claimed; that shape is covered by the command_name_expr pattern
; above instead).
(invokation_expression) @call
