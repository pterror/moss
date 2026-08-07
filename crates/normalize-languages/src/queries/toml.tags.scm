; TOML structure: tables/array tables as containers, pairs as variables.
; Inline table inner pairs are filtered out via node_name() in Rust.
;
; arborium-toml 2.17.0's grammar has no named fields at all (node-types.json
; shows an empty `fields` object for every node below) — the header key of a
; `table`/`table_array_element`, and the key of a `pair`, is whichever of
; three siblings appears: `bare_key` ([foo]), `quoted_key` (["quoted
; table"], "quoted key" = 1), or `dotted_key` ([tool.poetry], a.b.c = 1).
; All three are common, real-world TOML (Cargo.toml itself uses dotted table
; headers like [workspace.dependencies] and [profile.dev.build-override]);
; matching only `bare_key` silently dropped every dotted/quoted name.

(table
  [(bare_key) (quoted_key) (dotted_key)] @name) @definition.class

(table_array_element
  [(bare_key) (quoted_key) (dotted_key)] @name) @definition.class

(pair
  [(bare_key) (quoted_key) (dotted_key)] @name) @definition.var
