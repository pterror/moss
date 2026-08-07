; JSON key-value pairs as symbols.
; All pairs are captured as definition.var; container nesting is derived
; from the AST structure (pair > object > pair).
;
; The key is captured on the whole `string` node (quotes included), not on
; `string_content`, for two reasons verified against real parse output
; (`normalize syntax ast`/`normalize syntax query`):
;
;   1. Empty-string keys (`"": value`) parse as a `string` node with NO
;      `string_content` child at all -- the grammar only emits that child
;      for non-empty runs. A `(string (string_content) @name)` pattern
;      requires the child, so it silently drops every pair whose key is "".
;   2. Keys containing an escape sequence (`"a\nb"`) parse as a `string`
;      node with *multiple* `string_content` children (one per literal run
;      around each escape_sequence). A per-child pattern produces one
;      separate query match per run, which (a) yields duplicate
;      @definition.var matches for the same pair and (b) each match's
;      @name only ever holds one run ("a" or "b"), never the full key.
;
; Capturing the whole `string` node sidesteps both: exactly one match per
; pair, and json.rs's `node_name()` derives the actual name by slicing
; between the node's own start/end quotes rather than depending on child
; structure.
(pair
  key: (string) @name) @definition.var
