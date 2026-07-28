; C# imports query
; @import       — the entire using directive (for line number)
; @import.path  — the namespace or type path
; @import.alias — alias name (for `using Alias = Namespace;`)
;
; `using_directive`'s unlabeled path child is the `type` supertype
; (identifier / qualified_name / generic_name / alias_qualified_name, per
; arborium-c-sharp 2.17.0's node-types.json); the `name:` field, when
; present, holds the ALIAS identifier for `using Alias = Namespace;`. Two
; fixes are required on the plain (non-alias) patterns below, both regression
; guards against duplicate @import records for a single using directive:
;   1. The trailing `.` anchor: without it, the plain "using Namespace;"
;      pattern is unconstrained and matches ANY identifier/qualified_name
;      child, including the alias identifier itself in `using Sys = System;`
;      (producing a spurious @import.path = "Sys" in addition to "System").
;   2. The leading `!name` negation: without it, the plain pattern ALSO
;      matches aliased directives (correctly picking out just the real path
;      thanks to the anchor above) IN ADDITION TO the dedicated alias
;      pattern below also matching — i.e. every aliased import produced TWO
;      @import records (one without @import.alias, one with) instead of one.
;      `!name` excludes any using_directive that has a `name:` field at all,
;      so the plain patterns only fire for genuinely non-aliased directives.
; Since the path is always the LAST identifier/qualified_name/generic_name
; child regardless of `static`/`global`-ness, one anchored pattern per
; path-node kind covers the plain, `static`, and `global` forms uniformly —
; there is deliberately no separate "static"-specific variant (that was the
; redundant, duplicate-producing form this file used to have, mirroring the
; java.imports.scm fix from the same methodology sweep).

; using Namespace; / using static Namespace; / global using Namespace;
(using_directive
  !name
  (identifier) @import.path .) @import

; using Fully.Qualified.Namespace; / using static Fully.Qualified.Type;
(using_directive
  !name
  (qualified_name) @import.path .) @import

; using static List<T>; (bare generic type, no namespace qualifier — legal
; but unusual; only reachable via `using static`, since a plain `using`
; directive names a namespace, not a generic type)
(using_directive
  !name
  (generic_name) @import.path .) @import

; using global::System; (extern-alias-qualified import with no further
; qualification — a distinct top-level `alias_qualified_name` node,
; different from the far more common `global::System.Linq` case where the
; `alias_qualified_name` is nested inside an outer `qualified_name` and
; already covered by the qualified_name pattern above)
(using_directive
  !name
  (alias_qualified_name) @import.path .) @import

; using Alias = Namespace;
(using_directive
  name: (identifier) @import.alias
  (identifier) @import.path) @import

; using Alias = Fully.Qualified.Namespace;
(using_directive
  name: (identifier) @import.alias
  (qualified_name) @import.path) @import

; using Alias = List<T>; (generic alias target)
(using_directive
  name: (identifier) @import.alias
  (generic_name) @import.path) @import
