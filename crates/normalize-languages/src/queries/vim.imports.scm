; Vim script imports query
; @import       — the entire source/runtime statement (for line number)
; @import.path  — the file being sourced
;
; Both statements have an optional `bang` child (`source!`, `runtime!`).
; That child sits at the same tree depth as the path, so an unconstrained
; `(_)` child pattern captures it too, producing a spurious @import.path
; whose text is literally "!" — verified via `normalize syntax query` on a
; `source! file.vim` / `runtime! plugin/*.vim` probe. Match the specific
; node types/fields instead of `(_)` to avoid this.

; source file.vim  — `file` is a field of type `filename`.
(source_statement
  file: (filename) @import.path) @import

; runtime path/to/file.vim  — no field; the paths live under a `filenames`
; wrapper node (one or more `filename` children), a sibling of the
; optional `bang` node.
(runtime_statement
  (filenames
    (filename) @import.path)) @import
