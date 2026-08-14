; Vue CFG query
; Captures control flow nodes for CFG construction.
; See normalize-cfg for the full capture vocabulary.
; Verified against arborium Vue grammar node types using real fixtures.
;
; Vue uses directive attributes (v-if, v-else-if, v-else, v-for) for
; template-level control flow. These are attributes on elements, not
; statement nodes. JavaScript logic inside <script> is handled by the JS/TS
; grammar (intentionally out of scope here, matching the svelte.cfg.scm
; precedent — this file targets only the template-directive layer).
;
; This grammar has no `directive_argument` node type. The directive's value
; (the condition expression / for-loop head) is raw unparsed text sitting
; at `(directive_attribute (quoted_attribute_value (attribute_value)))`, not
; a parsed expression node — Vue's template grammar doesn't parse the
; expression inside `v-if="..."` as JS/TS. `v-else`/`v-else-if` are
; identified by the directive name text via `#match?`, since the grammar
; does not give them distinct node types from a plain `v-if` directive.
;
; A directive can sit on either of two structurally distinct tag node
; types — `start_tag` (an element with a body, `<div v-if="x">...</div>`)
; or `self_closing_tag` (`<Component v-if="x" />` — confirmed via
; node-types.json that both allow the identical
; attribute/directive_attribute/tag_name children, and both were verified
; empty of any built-in relationship: matching only `start_tag` silently
; dropped every directive on a self-closing element, which — since Vue
; components and void-like elements (`<slot>`, custom components) are
; conventionally self-closing — is a common, not a rare, real-world shape.
; Every pattern below is therefore duplicated for both tag kinds.

; ---------------------------------------------------------------------------
; v-if / v-else-if / v-else (branch via directives)
; ---------------------------------------------------------------------------

; v-if directive on an element with a body
(element
  (start_tag
    (directive_attribute
      (directive_name) @_d
      (quoted_attribute_value
        (attribute_value) @cfg.branch.condition)
      (#match? @_d "^v-if$")))
) @cfg.branch

; v-if directive on a self-closing element
(element
  (self_closing_tag
    (directive_attribute
      (directive_name) @_d
      (quoted_attribute_value
        (attribute_value) @cfg.branch.condition)
      (#match? @_d "^v-if$")))
) @cfg.branch

; v-else-if directive on an element with a body
(element
  (start_tag
    (directive_attribute
      (directive_name) @_d
      (quoted_attribute_value
        (attribute_value) @cfg.branch.condition)
      (#match? @_d "^v-else-if$")))
) @cfg.branch

; v-else-if directive on a self-closing element
(element
  (self_closing_tag
    (directive_attribute
      (directive_name) @_d
      (quoted_attribute_value
        (attribute_value) @cfg.branch.condition)
      (#match? @_d "^v-else-if$")))
) @cfg.branch

; v-else directive on an element with a body (no condition)
(element
  (start_tag
    (directive_attribute
      (directive_name) @_d
      (#match? @_d "^v-else$")))
) @cfg.branch

; v-else directive on a self-closing element (no condition)
(element
  (self_closing_tag
    (directive_attribute
      (directive_name) @_d
      (#match? @_d "^v-else$")))
) @cfg.branch

; ---------------------------------------------------------------------------
; v-for directive (loop over collection)
; ---------------------------------------------------------------------------

(element
  (start_tag
    (directive_attribute
      (directive_name) @_d
      (quoted_attribute_value
        (attribute_value) @cfg.loop.condition)
      (#match? @_d "^v-for$")))
) @cfg.loop

(element
  (self_closing_tag
    (directive_attribute
      (directive_name) @_d
      (quoted_attribute_value
        (attribute_value) @cfg.loop.condition)
      (#match? @_d "^v-for$")))
) @cfg.loop
