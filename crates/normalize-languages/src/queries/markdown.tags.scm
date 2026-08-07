; Markdown sections (heading + content)

; ATX-style heading: `# Heading` ... `###### Heading`. `heading_content` is
; the `inline` field directly.
(section
  (atx_heading
    (inline) @name)) @definition.heading

; Setext-style heading: `Heading\n=======` (level 1, `setext_h1_underline`)
; or `Heading\n-------` (level 2, `setext_h2_underline`). Unlike ATX headings,
; the grammar nests the heading text inside a `paragraph` node (the
; `heading_content` field's only allowed type) rather than exposing `inline`
; as a direct child of the heading itself.
;
; Deliberately anchored to `setext_heading` itself, not to the enclosing
; `section` (contrast the ATX pattern above): unlike an ATX heading — which
; this grammar always gives its own dedicated `section` — a setext heading
; only gets its own `section` when it happens to be the first block in that
; section. Anywhere else (e.g. a setext heading following a paragraph, or a
; second setext heading immediately after another with no body between
; them) it is just another sibling child of whatever `section` is already
; open. Anchoring to the enclosing `section` there would (a) report a body
; span that starts before the heading, at the section's true start, and
; (b) collide with that same section's own ATX/setext `@definition.heading`
; match, since both would resolve to the identical `section` node. Anchoring
; to `setext_heading` directly keeps every match's span tight and unique,
; at the cost of not modeling a "body" for setext headings the way ATX
; headings get one — an honest reflection of what this grammar structurally
; provides, not a limitation of the query.
(setext_heading
  (paragraph
    (inline) @name)) @definition.heading

; Note on capabilities: this grammar (arborium-markdown) does not ship a
; split inline sub-grammar — `inline` nodes are opaque leaves (aside from a
; handful of anonymous punctuation tokens for characters like `#`/`*`/`[`).
; Links, images, emphasis/strong, and autolinks are therefore not
; structurally distinguishable and cannot be captured as separate
; @definition/@reference entries; only the raw heading text is available.
; Likewise `list`/`list_item`, `pipe_table`, and `link_reference_definition`
; are not "definitions" in the symbol sense for this language and are
; intentionally left uncaptured.
