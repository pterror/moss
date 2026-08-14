; Line comments: `// ...` and doc line comments `/// ...`. `doc_comment` is a nested
; "doc" field on `line_comment` for the `///` form (verified via `normalize syntax
; ast`) — captured only via the wrapper here, so a `///` comment isn't double-counted
; against its own nested `doc_comment` child.
(line_comment) @decoration

; Block comments: `/* ... */` and doc block comments `/** ... */`. `doc_comment` is a
; nested "doc" field on `block_comment` for the `/**` form, same reasoning as above.
; A plain (non-doc) `/* ... */` block comment has no nested `doc_comment` at all, so
; without this pattern it was silently dropped entirely.
(block_comment) @decoration
