; Fish shell imports query
; @import       — the entire source command (for line number)
; @import.path  — the file path being sourced

; `argument` is a `multiple` field on `command` — `source file.fish arg1 arg2`
; (fish passes trailing words as positional $argv to the sourced script, a
; real idiom mirroring bash's `source file.sh arg1 arg2`) parses as three
; sibling `argument` nodes. Without the `.` anchor immediately after `name:`,
; this pattern matches once per argument, producing spurious @import.path
; captures for `arg1`/`arg2` alongside the real path — confirmed via
; `normalize syntax query` against a probe. The anchor restricts the match to
; the first argument only, mirroring the already-fixed bash.imports.scm.

; source file.fish
(command
  name: (word) @_cmd
  .
  argument: (_) @import.path
  (#eq? @_cmd "source")) @import

; . file.fish (legacy alias for `source`, still a working builtin in fish
; 4.8.0 — confirmed via `fish -c '. /path/to/file.fish'` and `fish -c 'type .'`)
(command
  name: (word) @_dot
  .
  argument: (_) @import.path
  (#eq? @_dot ".")) @import
