; Nginx imports query
; @import       — the entire include directive (for line number)
; @import.path  — the path pattern being included

; Grammar quirk (verified via `normalize syntax ast` — arborium-nginx 2.17.0):
; an unquoted include argument is NOT always a single `param` node. A glob
; character (`*`) splits the argument into multiple sibling `param` nodes with
; no wrapping node spanning them, e.g. `include /etc/nginx/conf.d/*.conf;`
; parses as three separate `param` siblings: "/etc/nginx/conf.d/", "*", ".conf".
; There is no query-only way to join sibling node text into one capture, so an
; unanchored `(param) @import.path` produces one spurious match PER fragment —
; three bogus "imports" (including a bare "*") for a single `include` line, one
; of the most common nginx idioms in the wild (default nginx.conf ships
; `include /etc/nginx/conf.d/*.conf;`). The `.` anchor below restricts the
; match to the *first* param only, so a plain (non-glob) include still captures
; its full, correct path, and a glob include captures its literal prefix
; (e.g. "/etc/nginx/conf.d/") instead of three garbage entries. The glob
; suffix (`*.conf`) is lost in this case — a real but unavoidable limitation
; of the grammar's tokenization, not something fixable at the query layer.
(simple_directive
  name: (directive) @_name
  .
  (param) @import.path
  (#eq? @_name "include")) @import
