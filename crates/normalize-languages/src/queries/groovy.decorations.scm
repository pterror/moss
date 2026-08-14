(annotation) @decoration

(comment) @decoration

; GroovyDoc comments (`/** ... */`) are a distinct `extra` node type from
; plain `comment` in this grammar — not a subtype of it — so they need their
; own clause. Confirmed via `normalize syntax ast`: `groovy_doc` and `comment`
; are sibling node kinds, both `extra: true`.
(groovy_doc) @decoration
