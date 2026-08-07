; CSS imports query
; @import       — the entire @import statement (for line number)
; @import.path  — the stylesheet path being imported

; @import "file.css";
(import_statement
  (string_value) @import.path) @import

; @import url("file.css");
(import_statement
  (call_expression
    (arguments
      (string_value) @import.path))) @import

; @import url(file.css);  (bare URL without quotes)
(import_statement
  (call_expression
    (arguments
      (plain_value) @import.path))) @import

; NOT HANDLED — grammar limitation, not a query gap: `@import "x.css"
; layer(name);` and `@import "x.css" supports(display: flex);` (CSS Cascade
; Layers / conditional-import syntax) both produce ERROR nodes in
; arborium-css 2.17.0 (confirmed via `normalize syntax ast` — the grammar's
; `import_statement` accepts a bare `keyword_query` like `layer`/`supports`
; but not the following parenthesized argument, which the grammar has no
; rule for). The `@import.path` string itself still matches correctly in
; these cases since it comes before the malformed suffix; only the
; layer/supports condition is unparseable.
