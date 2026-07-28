; Elixir imports query
; @import       — the entire call expression (for line number)
; @import.path  — the module alias/atom path
;
; In the tree-sitter-elixir grammar, alias/import/use/require are `call` nodes.
; The keyword is `target: (identifier)`, and the module is an `(alias)` child.

; alias Foo.Bar
(call
  target: (identifier) @_keyword
  (#eq? @_keyword "alias")
  (arguments (alias) @import.path)) @import

; import Foo.Bar
(call
  target: (identifier) @_keyword
  (#eq? @_keyword "import")
  (arguments (alias) @import.path)) @import

; use Foo.Bar
(call
  target: (identifier) @_keyword
  (#eq? @_keyword "use")
  (arguments (alias) @import.path)) @import

; require Foo.Bar
(call
  target: (identifier) @_keyword
  (#eq? @_keyword "require")
  (arguments (alias) @import.path)) @import

; Multi-alias/import/use/require form: `alias Foo.{Bar, Baz}` — extremely
; common idiomatic Elixir (also grammar-legal, if rarer, with import/use/
; require, verified via `normalize syntax query`). A dotted path followed by
; a brace-list of names parses as `dot left: (alias) right: (tuple (alias)
; ...))`, not as an `(alias)` node directly under `arguments` — the four
; patterns above never match this shape at all. Each bare name inside the
; tuple is captured individually (best-effort partial reference: "Bar",
; "Baz", not the reconstructed "Foo.Bar"/"Foo.Baz" — query-only extraction
; has no way to concatenate the "Foo." prefix back onto each tuple member,
; matching the precedent set for Ruby's dynamic-superclass capture).
(call
  target: (identifier) @_keyword
  (#match? @_keyword "^(alias|import|use|require)$")
  (arguments
    (dot
      right: (tuple
        (alias) @import.path)))) @import

; Dot-qualified single form: `alias __MODULE__.Sub` — the qualifying
; left-hand side is not itself an `alias` token (e.g. `__MODULE__` lexes as
; a plain `identifier`, verified via `normalize syntax query`), so the whole
; expression parses as `dot left: (identifier) right: (alias)` rather than a
; bare `(alias)` under `arguments`. Captures the right-hand segment only
; (best-effort partial, same rationale as the multi-alias form above).
(call
  target: (identifier) @_keyword
  (#match? @_keyword "^(alias|import|use|require)$")
  (arguments
    (dot
      right: (alias) @import.path))) @import
