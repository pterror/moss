; AWK (GAWK) imports query
; @import       — the entire directive (for line number)
; @import.path  — the file path being included/loaded
;
; `directive` (per node-types.json) has no field distinguishing which
; keyword introduced it — `@include`, `@load`, and `@namespace` are all
; the same `directive > string` shape, differing only in the literal
; (anonymous) keyword token. An earlier unscoped `(directive (string)
; @import.path) @import` matched ALL THREE, including `@namespace "ns"`
; (a gawk namespace declaration, not an import/include) — verified via
; `normalize syntax query --show-source` that `@namespace "mylib"` was
; being captured as if it were an import path. Scoped to the two literal
; keywords that actually name a file/extension to load.

; @include "file.awk"
(directive "@include" (string) @import.path) @import

; @load "extension"
(directive "@load" (string) @import.path) @import
