; Bash imports query
; @import       — the entire source command (for line number)
; @import.path  — the file path being sourced

; `argument` is a `multiple` field on `command` — `source file.sh arg1 arg2`
; (bash passes trailing words as positional args to the sourced script, a
; real idiom) parses as three sibling `argument` nodes. Without the `.`
; anchor immediately after `name:`, this pattern matches once per argument,
; producing spurious @import.path captures for `arg1`/`arg2` alongside the
; real path. The anchor restricts the match to the first argument only.

; source file.sh
(command
  name: (command_name) @_cmd (#eq? @_cmd "source")
  .
  argument: (_) @import.path) @import

; . file.sh (POSIX dot command)
(command
  name: (command_name) @_dot (#eq? @_dot ".")
  .
  argument: (_) @import.path) @import
