; Vue complexity query
; @complexity — nodes that increase cyclomatic complexity
; @nesting — nodes that increase nesting depth

; Complexity nodes: only the branch/loop directives (v-if/v-else-if/v-else/
; v-for) count — matching vue.cfg.scm's branch/loop capture set and the
; #match?-on-directive_name pattern it established (this grammar gives
; every v-* directive the same generic `directive_attribute` node type, so
; distinguishing them requires matching on the directive's name text, not
; its node kind — confirmed via `normalize syntax ast`/node-types.json).
; `(directive_attribute) @complexity` unfiltered was a real bug: it counted
; every v-bind/v-on/v-model/v-slot/v-show usage as a decision point too —
; on this crate's own sample.vue that inflated 4 genuine branch/loop
; directives to 8 "complexity" nodes by also sweeping in :key, :href,
; @click, and :disabled (verified via `normalize syntax query`).
;
; Matched for both `start_tag` (elements with a body) and `self_closing_tag`
; (`<Component v-if="x" />`) — see vue.cfg.scm's header comment for why
; both are needed: they're structurally distinct node types with identical
; directive_attribute/attribute/tag_name children, and self-closing is the
; conventional form for Vue components, so a start_tag-only match silently
; drops directives on every self-closing component/element.
(element
  (start_tag
    (directive_attribute
      (directive_name) @_d
      (#match? @_d "^v-(if|else-if|else|for)$")))
) @complexity

(element
  (self_closing_tag
    (directive_attribute
      (directive_name) @_d
      (#match? @_d "^v-(if|else-if|else|for)$")))
) @complexity

(interpolation) @complexity

; Nesting nodes: same branch/loop-only scoping as complexity above.
(element
  (start_tag
    (directive_attribute
      (directive_name) @_d
      (#match? @_d "^v-(if|else-if|else|for)$")))
) @nesting

(element
  (self_closing_tag
    (directive_attribute
      (directive_name) @_d
      (#match? @_d "^v-(if|else-if|else|for)$")))
) @nesting

(element) @nesting
