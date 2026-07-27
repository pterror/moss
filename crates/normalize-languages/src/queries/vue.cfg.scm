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

; ---------------------------------------------------------------------------
; v-if / v-else-if / v-else (branch via directives)
; ---------------------------------------------------------------------------

; v-if directive on an element
(element
  (start_tag
    (directive_attribute
      (directive_name) @_d
      (quoted_attribute_value
        (attribute_value) @cfg.branch.condition)
      (#match? @_d "^v-if$")))
) @cfg.branch

; v-else-if directive on an element
(element
  (start_tag
    (directive_attribute
      (directive_name) @_d
      (quoted_attribute_value
        (attribute_value) @cfg.branch.condition)
      (#match? @_d "^v-else-if$")))
) @cfg.branch

; v-else directive on an element (no condition)
(element
  (start_tag
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
