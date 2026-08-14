(comment) @decoration

; Trailing type/codegen annotations, e.g. `(cpp.type = "std::string")` on a
; field, struct, or const declaration -- pervasive in real-world Thrift IDL
; for customizing per-language codegen. Mirrors the convention in other
; languages' decorations.scm (e.g. rust's `attribute_item`, java's
; `annotation`) of treating attribute/annotation-like nodes as decorations
; alongside comments.
(annotation_definition) @decoration

; Facebook fbthrift-style prefix annotations, e.g. `@fb.Foo` before a
; struct/service/field definition.
(fb_annotation_definition) @decoration
