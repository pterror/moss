; Java imports query
; @import       — the entire import statement (for line number)
; @import.path  — the fully-qualified class or package path
; @import.glob  — wildcard marker (presence means is_wildcard=true)
;
; `import_declaration`'s children (per arborium-java node-types.json) are
; `[asterisk, identifier, scoped_identifier]`, in either the plain form or
; prefixed with the `static` keyword. The `.` anchor after `scoped_identifier`
; below is required: without it, the plain "import pkg.Class;" pattern is
; unconstrained and ALSO matches every wildcard and `static` import (since it
; only requires *some* scoped_identifier child, not the sole one) — producing
; duplicate @import matches per statement. static-ness doesn't change which
; child holds the path, so there is deliberately no separate "static" variant
; of these two patterns (that was the redundant, duplicate-producing form
; this file used to have).

; import pkg.Class; / import static pkg.Class.method;
(import_declaration
  (scoped_identifier) @import.path .) @import

; import pkg.*;  (wildcard) / import static pkg.Class.*;
(import_declaration
  (scoped_identifier) @import.path
  (asterisk) @import.glob) @import

; import Foo; (bare single-segment import, no package qualifier)
; `import_declaration`'s children allow a plain `identifier` in addition to
; `scoped_identifier`, and the grammar parses it cleanly even though
; importing from the unnamed/default package is not legal Java — this is a
; real, if rare, grammar-legal form (e.g. tooling that generates or fuzzes
; source) that the scoped-only patterns above silently dropped.
(import_declaration
  (identifier) @import.path) @import
