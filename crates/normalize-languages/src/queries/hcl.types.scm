; HCL types query
; @type — type constraint expressions in variable blocks
;
; HCL (Terraform) uses type constraints in `variable` blocks:
;   variable "example" {
;     type = string
;     type = list(string)
;     type = object({ name = string })
;   }
;
; In the tree-sitter grammar, `type = string` is an `attribute` where the
; first child identifier is literally "type" and the second child is an
; expression. We capture the expression as the type reference.
;
; Scoped to `variable` blocks specifically (not just "any attribute named
; `type`"): `type` is also an ordinary, unrelated string-valued attribute name
; on many real resource types — e.g. `aws_route53_record`'s
; `type = "A"` (the DNS record type) or `aws_lb_target_group`'s
; `type = "instance"`. Without this scoping every such resource attribute was
; misidentified as a type-constraint reference (a real false positive,
; confirmed via `normalize syntax query` against a fixture combining a
; `variable` block with an `aws_route53_record` resource). HCL/arborium-hcl
; 2.17.0's grammar has no named fields at all (`block`/`attribute` both have
; an empty `fields` object in node-types.json), so this is expressed
; positionally rather than via a field constraint.

; Type constraint attribute: type = string / type = list(string) / type = object({...})
; — only inside a `variable` block.
(block
  (identifier) @_block_type (#eq? @_block_type "variable")
  (body
    (attribute
      (identifier) @_key (#eq? @_key "type")
      (expression) @type)))
