; Zig decorations query
; Captures documentation comments.
;
; Verified against arborium-zig 2.17.0's node-types.json: `doc_comment`
; (`///`, attached to the following declaration) and `container_doc_comment`
; (`//!`, module/container-level, e.g. a file's top-of-file doc block) are
; distinct node types — the prior version only matched `doc_comment`,
; silently dropping every `//!` module doc comment.

(doc_comment) @decoration

(container_doc_comment) @decoration
