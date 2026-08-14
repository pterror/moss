; Clojure imports query
; @import       — the entire require/import form (for line number)
; @import.path  — the namespace/class being required or imported
; @import.alias — namespace alias (:as alias)
;
; Verified against arborium-clojure 2.17.0's grammar via `normalize syntax
; ast`/`normalize syntax query` — never guessed from node-types.json alone.
;
; Two previously-silent bugs fixed here:
;
; 1. `(vec_lit (sym_lit) @import.path)` (no `.` anchor) matched *every*
;    `sym_lit` child of the require vector, not just the leading namespace —
;    so `[other.ns :as o]` captured both "other.ns" (correct) AND "o" (the
;    alias symbol, wrongly captured a second time as an @import.path). All
;    namespace-path patterns below anchor with `.` to bind only the first
;    child.
; 2. The bare top-level `(require '[ns :as alias])` form — by far the most
;    common way to require a namespace outside an `ns` form — was completely
;    unmatched. The query only handled the un-quoted `(require [...])` shape,
;    but real Clojure always quotes the vector at the top level (`'[...]`),
;    which parses as `quoting_lit` wrapping `vec_lit`, not a bare `vec_lit`
;    child of `list_lit`. Confirmed via `normalize syntax ast` on
;    `(require '[namespace.core :as nc])`.
;
; `(import ...)` (Java interop) was entirely unhandled — no query pattern
; existed for it at all, silently dropping every Java class import from
; extraction. Added below in both its forms: `(import (pkg Class1 Class2))`
; and `(import 'pkg.Class)`/`(import pkg.Class)`. Likewise the `(:import ...)`
; clause nested inside `(ns ...)`.

; (require '[namespace.core :as nc])
; (require '[a.b :as ab] '[c.d :as cd])  — multiple quoted vectors
(list_lit
  (sym_lit) @_f (#eq? @_f "require")
  (quoting_lit
    (vec_lit
      .
      (sym_lit) @import.path
      (kwd_lit) @_as (#eq? @_as ":as")
      .
      (sym_lit) @import.alias))) @import

; (require '[namespace.core]) — no :as alias
(list_lit
  (sym_lit) @_f (#eq? @_f "require")
  (quoting_lit
    (vec_lit
      .
      (sym_lit) @import.path))) @import

; (require [namespace.core :as nc]) — unquoted (rarer, but the grammar
; allows it and the pre-existing query already handled the unquoted shape
; inside `ns`, so keep it for top-level `require` too).
(list_lit
  (sym_lit) @_f (#eq? @_f "require")
  (vec_lit
    .
    (sym_lit) @import.path
    (kwd_lit) @_as (#eq? @_as ":as")
    .
    (sym_lit) @import.alias)) @import

(list_lit
  (sym_lit) @_f (#eq? @_f "require")
  (vec_lit
    .
    (sym_lit) @import.path)) @import

; (ns my.ns (:require [other.ns :as o] [other.ns2 :refer [x y]]))
(list_lit
  (sym_lit) @_f (#eq? @_f "ns")
  (list_lit
    (kwd_lit) @_req (#eq? @_req ":require")
    (vec_lit
      .
      (sym_lit) @import.path
      (kwd_lit) @_as (#eq? @_as ":as")
      .
      (sym_lit) @import.alias))) @import

(list_lit
  (sym_lit) @_f (#eq? @_f "ns")
  (list_lit
    (kwd_lit) @_req (#eq? @_req ":require")
    (vec_lit
      .
      (sym_lit) @import.path))) @import

; (import (java.util Date List)) — package-grouped form: leading sym_lit is
; the package (@import.path), remaining sym_lit siblings are class names
; imported from it (@import.name — mirrors how `from x import a, b`-style
; multi-name imports are modeled elsewhere: FlatImport pairs a shared
; `module`/path with one `name` per capture). Scoped to a nested list_lit
; directly under `import`/`:import` so it can't over-match arbitrary
; 2+-symbol lists like `(+ x y)` or `(extend-protocol IFoo ...)` — verified
; via `normalize syntax query` that an unscoped version of this pattern does
; over-match.
(list_lit
  (sym_lit) @_f (#eq? @_f "import")
  .
  (list_lit
    .
    (sym_lit) @import.path
    (sym_lit) @import.name)) @import

; No `.` anchor here (unlike the top-level `import` form below): a `:import`
; clause commonly mixes multiple entries, e.g.
; `(:import (java.util List Map) java.io.InputStream)`, where the bare-class
; entry is NOT the immediate next sibling after the `:import` keyword.
; Anchoring on immediate adjacency (as first written) silently dropped every
; entry but the first — confirmed via `normalize syntax query` against a
; `:import` clause with more than one entry.
(list_lit
  (kwd_lit) @_imp (#eq? @_imp ":import")
  (list_lit
    .
    (sym_lit) @import.path
    (sym_lit) @import.name))

; (import 'java.util.UUID) — quoted single fully-qualified class
(list_lit
  (sym_lit) @_f (#eq? @_f "import")
  .
  (quoting_lit
    (sym_lit) @import.path)) @import

; (import java.util.UUID) / (:import java.io.File) — bare fully-qualified
; class, either as a top-level `import` form or a bare entry in an `ns`
; form's `:import` clause.
(list_lit
  (sym_lit) @_f (#eq? @_f "import")
  .
  (sym_lit) @import.path) @import

(list_lit
  (kwd_lit) @_imp (#eq? @_imp ":import")
  (sym_lit) @import.path)
