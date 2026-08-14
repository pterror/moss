; Dart imports query
; @import       — the entire library import (for line number)
; @import.path  — the URI string

; import 'dart:collection';
(library_import
  (import_specification
    (uri
      (string_literal) @import.path))) @import

; import 'dart:collection' show Foo;
(library_import
  (import_specification
    (configurable_uri
      (uri
        (string_literal) @import.path)))) @import

; export 'uri';
(library_export
  (configurable_uri
    (uri
      (string_literal) @import.path))) @import

; part 'uri'; — splits a library across files sharing the same scope. A
; distinct node type from library_import/library_export, entirely
; unmatched before — a common idiom for large Dart libraries.
(part_directive
  (uri
    (string_literal) @import.path)) @import

; part of 'uri'; — the file's declaration of which library it belongs to.
; part_of_directive can reference the owning library either by URI (modern
; form, shown here) or by a dotted library name (legacy `part of
; my.library.name;`, which has no `uri` child — the grammar genuinely
; cannot express that form as a path, so it's left unmatched rather than
; fabricating one).
(part_of_directive
  (uri
    (string_literal) @import.path)) @import
