<!--
Completeness-matrix fixture for markdown.tags.scm.

One construct per node-type variant found in `node-types.json` for
arborium-markdown 2.17.0 that `markdown.tags.scm` constrains against
(`atx_heading` / `setext_heading`, both allowed by the grammar as a
`section`'s heading), plus a dedicated NEGATIVE section for constructs that
must not match. Each construct is commented with which variant it exercises.

Grammar-capability note: this grammar (arborium-markdown, no split inline
sub-grammar bundled) never turns `inline` into a structured tree — links,
images, emphasis/strong, and autolinks are opaque byte spans inside
`inline`, distinguishable from each other only by re-scanning the raw text,
not by node kind. There is therefore no field-variant matrix to enumerate
for those constructs the way there is for e.g. Rust's `call_expression`
function field; see `markdown.tags.scm`'s header comment.
-->

<!-- ATX heading variants: one marker per level 1-6 (atx_h1_marker .. atx_h6_marker). -->

# ATX level 1

## ATX level 2

### ATX level 3

#### ATX level 4

##### ATX level 5

###### ATX level 6

<!-- ATX closing sequence: trailing `#`s after the content are not stripped
     by this grammar version — they remain part of the `inline` node's byte
     range (see markdown.tags.scm's comment on the `atx_heading` pattern).
     Documented behavior, not a query gap: the closing `##` below is real
     content per the CST, and the query captures it as such. -->

## ATX level 2 with closing sequence ##

<!-- Setext heading variants: level 1 (`=`, setext_h1_underline) and level 2
     (`-`, setext_h2_underline). Anchored directly to `setext_heading`, not
     to the enclosing `section` — see markdown.tags.scm for why. -->

Setext level 1
==============

Setext level 2
--------------

<!-- Setext heading immediately followed by another setext heading, with no
     body content between them: per this grammar, both land as siblings
     inside the SAME enclosing `section` rather than each getting its own
     nested `section` (contrast: two consecutive ATX headings always split
     into nested `section`s). Confirms the `setext_heading`-anchored query
     still gives each one its own tight, non-overlapping @definition.heading
     span even though the grammar doesn't give them their own containers. -->

Back to back A
==============
Back to back B
--------------

<!-- Setext heading following ordinary paragraph content (not the first
     block in its section): the CST nests it as a sibling of that content
     inside the *same* `section`, not a fresh one. Real-world shape (e.g. a
     hand-written "License" section using a setext-style divider after
     prose) — exercised more fully in sample.md's "License" section. -->

## Preceding content

Some prose before the divider.

Trailing setext divider
------------------------

<!-- Heading nested inside a block quote: `block_quote`'s children include
     `section`, so a heading inside a quoted callout is still just a
     `section` → the query matches it the same as any other, no special
     case needed. -->

> ### Heading inside a block quote
>
> Quoted body content.

<!-- Heading nested inside a list item: `list_item`'s children include
     `section` too. -->

- Intro item

  #### Heading inside a list item

  Body text for that heading.

<!-- ============================= NEGATIVE ============================= -->
<!-- Constructs that must NOT produce a @definition.heading match. -->

<!-- Seven `#` characters exceed the grammar's max heading level (only
     atx_h1_marker..atx_h6_marker exist) — parses as an ordinary paragraph,
     not an atx_heading. -->

####### Not a heading: seven hashes

<!-- Empty ATX heading: `heading_content` is an optional field (a bare `#`
     with no text has no `inline` child at all), so there is nothing for
     `(inline) @name` to bind to and the pattern correctly does not match. -->

#

<!-- A `#` that is not at the start of a line is inline text, not a marker. -->

Text with a stray # character mid-line, not a heading.

<!-- Thematic breaks (`---`, `***`, `___`) must never be mistaken for a
     setext underline: only `-`/`=` runs directly under a paragraph with no
     blank line between them count as setext_h1_underline/h2_underline.
     A `---` on its own (no preceding paragraph line) is thematic_break. -->

---

***

___

<!-- A fenced/indented code block's contents must never match as a heading,
     even when they look like one lexically. -->

```markdown
# This is inside a fenced code block, not a real heading.
```

    # This is inside an indented code block, not a real heading.

<!-- A pipe-table header row must never match as a heading. -->

| Not a heading | Also not |
|----------------|----------|
| cell           | cell     |
