; Objective-C imports query
; @import       — the entire #import or #include directive (for line number)
; @import.path  — the header file path (quotes/angles stripped by Rust)

; #import "Header.h"
; #import <Framework/Header.h>
; #include "file.h"
(preproc_include
  path: (_) @import.path) @import

; @import Foundation;              — Clang module import (no framework
;                                     `#import`, common in modern Xcode
;                                     projects with modules enabled)
; @import Foundation.NSString;     — dotted submodule import
;
; `module_import.path` is a `multiple: true` field of alternating
; identifier/`.` tokens directly under `module_import`, with NO wrapping
; node spanning the full dotted path (verified via node-types.json + probe;
; every other language's dotted-import handling in this workspace captures
; a single wrapper node — see java/python/rust .imports.scm — because that's
; what the consumer in normalize-deps/normalize-facts expects: it keeps only
; the FIRST @import.path bound per @import anchor, dropping any later ones
; sharing the same anchor). Capturing `(identifier) @import.path` here
; correctly yields the single segment for the common single-component case
; (`@import Foundation;`) and, for a dotted submodule import, yields just
; the leading/framework segment (`Foundation`, dropping `.NSString`) rather
; than the full path — an honest partial fix (0 coverage -> framework-level
; coverage) rather than fabricating a join the query layer can't express.
(module_import
  path: (identifier) @import.path) @import
