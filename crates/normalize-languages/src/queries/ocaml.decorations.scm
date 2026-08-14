(attribute) @decoration

; [@@deriving show, eq] / [@@inline] / [@@warning ...] — the "item attribute"
; form attached after a definition (`let`, `type`, `module`, …), distinct
; from `attribute` (the `[@attr]` form attached before an expression).
; `[@@deriving ...]` (ppx_deriving) is one of the most common attribute
; idioms in real-world OCaml and was entirely unmatched before this fix.
(item_attribute) @decoration

(floating_attribute) @decoration

(comment) @decoration
