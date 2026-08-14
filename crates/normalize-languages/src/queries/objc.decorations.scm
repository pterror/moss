(comment) @decoration

;; preproc_include covers both #include and #import in the ObjC grammar
(preproc_include) @decoration

;; #pragma mark - Section, #pragma once, etc. — the `#pragma mark` idiom in
;; particular is used pervasively in real ObjC code for editor-jump-bar
;; section headers. Previously uncaptured (verified via probe: 0 matches).
;; Ported from c.decorations.scm, which already covers this for plain C.
(preproc_call) @decoration

;; __attribute__((deprecated)), __attribute__((unused)), etc. — GCC/Clang
;; attribute syntax, verified present in the grammar as `attribute_specifier`
;; (both on declarations and on method_declaration selectors) via probe.
;; Mirrors swift.decorations.scm's `(attribute) @decoration` for the nearest
;; equivalent concept in a language this workspace already treats as a
;; decoration.
(attribute_specifier) @decoration
