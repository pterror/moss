; MATLAB imports query
; @import       — the entire import command (for line number)
; @import.path  — the package/class being imported

; import pkg.Class
; import pkg.*
;
; `command` has no fields at all (grammar declares `"fields": {}`) — its
; children are the terminal token types `command_name` and `command_argument`,
; not `(identifier)`/`argument:`.
(command
  (command_name) @_cmd (#eq? @_cmd "import")
  (command_argument) @import.path) @import
