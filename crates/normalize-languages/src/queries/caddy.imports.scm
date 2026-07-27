; Caddyfile imports query
; @import       — the entire import directive (for line number)
; @import.path  — the snippet reference being imported
;
; The tree-sitter-caddy grammar only models the parenthesized snippet-reference
; form of `import`, e.g. `import (common-headers)` — the node type is
; `directive_import` (not `import`), wrapping a `snippet_name` child that
; includes the parens.

; import (snippet-name)
(directive_import
  (snippet_name) @import.path) @import
