; Scheme imports query
; @import       — the entire import/require form (for line number)
; @import.path  — the library being imported

; (import (library name)) — plain form. R7RS import sets `(only ...)`,
; `(except ...)`, `(prefix ...)`, `(rename ...)` are excluded here (handled
; by the dedicated unwrapping pattern below) — without this exclusion the
; whole wrapper form (e.g. `(only (scheme base) car cdr)`) was captured as
; the "library path" instead of the actual library it wraps, verified via
; `normalize syntax query` against a probe file.
(list
  (symbol) @_f (#eq? @_f "import")
  .
  (_) @import.path
  (#not-match? @import.path "^\\((only|except|prefix|rename)\\b")) @import

; (import (only (library) name ...)) / (except ...) / (prefix ... p:) /
; (rename ...) — R7RS import sets. Unwrap to the underlying library form
; rather than capturing the wrapper (verified via `normalize syntax ast`:
; the wrapper is itself a `list` headed by the `only`/`except`/`prefix`/
; `rename` symbol, with the real library form as its next child).
(list
  (symbol) @_f (#eq? @_f "import")
  .
  (list
    .
    (symbol) @_wrap (#match? @_wrap "^(only|except|prefix|rename)$")
    .
    (_) @import.path)) @import

; (require 'library) / (require library)
(list
  (symbol) @_f (#eq? @_f "require")
  (_) @import.path) @import
