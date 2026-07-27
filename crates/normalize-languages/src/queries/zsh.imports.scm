; Zsh imports query
;
; IMPORTANT — UNVERIFIED AT RUNTIME, KNOWN-BROKEN GRAMMAR:
; This query is schema-correct (checked against arborium-zsh's node-types.json)
; but its runtime capture behavior is unverified: the vendored arborium-zsh
; grammar ships "externals": [] and no scanner.c (upstream tree-sitter-zsh has
; a ~92KB hand-written one), so real zsh source parses into ERROR-laden trees
; rather than proper `command` nodes. This is a packaging defect in
; bearcove/arborium (https://github.com/bearcove/arborium/issues/213), not
; fixable by a query rewrite here. See TODO.md for the full investigation and
; `normalize_languages::known_broken_grammar("zsh")` for the runtime-facing
; surface of this fact.
;
; @import       — the entire source command (for line number)
; @import.path  — the file path being sourced

; source file.zsh
(command
  name: (command_name) @_cmd (#eq? @_cmd "source")
  argument: (_) @import.path) @import

; . file.zsh (POSIX dot command)
(command
  name: (command_name) @_dot (#eq? @_dot ".")
  argument: (_) @import.path) @import
