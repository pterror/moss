; KDL nodes as symbols: every `node` carries an `identifier` as its name
; (bare word or quoted string — the quoted form wraps a nested `string`).
; A node with a `children` field (a `{ ... }` block) acts as a container,
; analogous to a TOML table; a node without one is a leaf entry, analogous
; to a TOML pair.

; `node`'s children (per node-types.json, all positional — none are named
; fields except `children:`) may include, before the name `identifier`: an
; optional `(type)` annotation (`(u8)my-node 5`), and/or `node_comment`
; tokens marking a KDL "slash-dash" comment (`/- disabled-node "x"` — the
; entire node is commented out, but the grammar still parses it as a live
; `node` with a `node_comment` child preceding the identifier, not an ERROR
; and not omitted from the tree). Verified via `normalize syntax ast`: an
; unanchored `(node (identifier) @name ...)` pattern matched slash-dash-
; disabled nodes identically to live ones, silently extracting commented-out
; config as real symbols. Anchoring `.` immediately before `(identifier)`
; excludes the `node_comment`-prefixed (disabled) case; a second, separately
; anchored `(type)`-then-`(identifier)` pair of patterns preserves typed-node
; support without reopening that gap (an unanchored `(type)? (identifier)`
; alternative would still let a leading `node_comment` slip through since
; `(type)?` is optional). A live *trailing* comment on the same line
; (`node "a" // comment`) does not break the anchor — verified via probe —
; since it appears after the identifier/args, not before.
;
; Known remaining limitation: this only excludes a node whose OWN header is
; slash-dashed. Per the KDL spec, `/-` in front of a container disables its
; entire subtree, but nested `node`s inside that container's `node_children`
; carry no `node_comment` marker of their own — only their disabled ancestor
; does. Excluding them would require an "ancestor has node_comment" check,
; which this codebase's query-predicate engine (`match?`/`not-match?`/`eq?`/
; `not-eq?` only, see `normalize_languages::satisfies_predicates`) cannot
; express, and tree-sitter's own query language has no ancestor predicate.
; A node nested inside a slash-dashed container is therefore still captured
; as a symbol today (verified via `variants.kdl`'s
; `disabled-top-level-container`/`disabled-typed-container` cases). Fixing
; this needs new predicate support in the query engine — out of scope for a
; query-file-only fix.

(node
  . (identifier) @name
  children: (node_children)) @definition.class

(node
  . (identifier) @name
  !children) @definition.var

; Typed variants: `(u8)my-node { ... }` / `(u8)my-node 5`
(node
  . (type) . (identifier) @name
  children: (node_children)) @definition.class

(node
  . (type) . (identifier) @name
  !children) @definition.var
