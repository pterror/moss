; YAML mapping keys as symbols.
;
; Verified against arborium-yaml 2.17.0's node-types.json (block_mapping_pair.key /
; flow_pair.key both allow the full flow_node child set: alias, anchor,
; double_quote_scalar, flow_mapping, flow_sequence, plain_scalar,
; single_quote_scalar, tag) and cross-checked with `normalize syntax query`
; against real parse output.
;
; Anchors/tags prefixing a scalar key (`&anchor key:`, `!!str key:`) don't wrap
; the scalar node — they're siblings within the same flow_node — so these
; patterns still match through them without a dedicated clause.

(block_mapping_pair
  key: (flow_node
    (plain_scalar
      (string_scalar) @name))) @definition.var

(block_mapping_pair
  key: (flow_node
    (double_quote_scalar) @name)) @definition.var

(block_mapping_pair
  key: (flow_node
    (single_quote_scalar) @name)) @definition.var

; Flow mapping pairs (`{key: value, ...}`) are a structurally distinct node
; (flow_pair, not block_mapping_pair) but the same conceptual "key" symbol.
; This was previously entirely unhandled — real-world density check found
; 400+ flow_pair keys in this repo's own docs/pnpm-lock.yaml alone (e.g.
; `resolution: {integrity: ..., engines: ...}`).

(flow_pair
  key: (flow_node
    (plain_scalar
      (string_scalar) @name))) @definition.var

(flow_pair
  key: (flow_node
    (double_quote_scalar) @name)) @definition.var

(flow_pair
  key: (flow_node
    (single_quote_scalar) @name)) @definition.var

; Deliberately NOT handled (verified via `normalize syntax query` that the
; grammar allows these but they don't fit the "symbol name" concept, and
; real-world usage is effectively zero — no occurrence anywhere in this
; repo's own YAML files):
;   - Complex keys: `? [a, b]` (flow_sequence) or `? {a: 1}` (flow_mapping) as
;     the explicit key — no single scalar name to extract.
;   - Block-scalar explicit keys: `? |\n  multi-line key\n: value` — key is
;     block_node > block_scalar, a multi-line blob, not a nameable symbol.
;
; Anchors (`&name`) and aliases (`*name`) are a genuine define/reference
; relationship the grammar supports (`anchor`/`alias` node types), but this
; codebase's tags pipeline (`tags_capture_to_kind` in normalize-facts)
; only turns `@definition.*` captures into symbols and drops all
; `@reference.*` captures for symbol-extraction purposes — there is no
; existing convention or consumer for a non-call def/reference pair (checked:
; no `@reference.var`/`@local.definition` precedent anywhere in this crate's
; query set). Adding capture clauses for them here would be dead weight with
; no consumer, so they're left unhandled rather than speculatively wired up.
