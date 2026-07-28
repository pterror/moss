; KDL nodes as symbols: every `node` carries an `identifier` as its name
; (bare word or quoted string — the quoted form wraps a nested `string`).
; A node with a `children` field (a `{ ... }` block) acts as a container,
; analogous to a TOML table; a node without one is a leaf entry, analogous
; to a TOML pair.

(node
  (identifier) @name
  children: (node_children)) @definition.class

(node
  (identifier) @name
  !children) @definition.var
