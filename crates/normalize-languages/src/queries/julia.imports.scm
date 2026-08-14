; Julia imports query
; @import       — the entire import/using statement (for line number)
; @import.path  — the module being imported
; @import.name  — a single selected name (using Pkg: foo, bar)
; @import.alias — alias for a module or a selected name (`as` clause)
;
; import_statement/using_statement have no named fields at all (node-types.json:
; empty fields object) — everything is positional. Verified via `normalize
; syntax ast`/`syntax query` against probe files that the direct children take
; one of four shapes:
;   - (identifier)                         import Pkg / using Pkg
;   - (import_path)                        import Pkg.Sub / using Pkg.Sub
;   - (import_alias identifier identifier) import Pkg as P
;     (import_alias import_path identifier) import Pkg.Sub as P
;   - (selected_import ...)                import Pkg: a, b / using Pkg: a, b
;     whose own first child is (identifier) or (import_path) — the module —
;     followed by one (identifier) per selected name, or an (import_alias)
;     when a selected name itself has an `as` clause (using Pkg: a as b).
;
; PRIOR BUG (fixed here): the previous query was
;   (import_statement (_) @import.path) @import
; Because `import Pkg: a, b` nests the entire "Pkg: a, b" in a single
; selected_import child, the old unanchored `(_)` matched that whole
; selected_import node as @import.path — producing a garbled path like
; "Statistics: mean, std" instead of "Statistics", and never emitting the
; @import.name capture the header comment promised. Confirmed via
; `normalize syntax query` this query is live (normalize-deps's
; collect_imports_from_query is query-first, only falling back to the
; Language-trait text-split extractor when the query yields nothing), so
; this corrupted every selective-import extraction in real Julia code —
; `using X: a, b` is a very common idiom.
;
; NOT covered: (interpolation_expression)/(macro_identifier) as a bare
; import path or alias target. node-types.json lists them as legal, but `$`
; interpolation and `@macro` syntax are only valid inside a `quote`/macro
; body — an import_statement/using_statement written directly in source
; (not inside a quote) can never contain one. Confirmed no probe combining
; interpolation/macro syntax with plain import/using produced anything
; other than a parse error or a non-import node. Not fabricated.

; --- plain: import Pkg / using Pkg.Sub -------------------------------------

(import_statement
  . (identifier) @import.path) @import

(import_statement
  . (import_path) @import.path) @import

(using_statement
  . (identifier) @import.path) @import

(using_statement
  . (import_path) @import.path) @import

; --- aliased: import Pkg as P / using Pkg.Sub as P -------------------------

(import_statement
  (import_alias
    . (identifier) @import.path
    (identifier) @import.alias)) @import

(import_statement
  (import_alias
    . (import_path) @import.path
    (identifier) @import.alias)) @import

(using_statement
  (import_alias
    . (identifier) @import.path
    (identifier) @import.alias)) @import

(using_statement
  (import_alias
    . (import_path) @import.path
    (identifier) @import.alias)) @import

; --- selective: import Pkg: a, b / using Pkg: a, b -------------------------

(import_statement
  (selected_import
    . (identifier) @import.path
    (identifier) @import.name)) @import

(import_statement
  (selected_import
    . (import_path) @import.path
    (identifier) @import.name)) @import

(using_statement
  (selected_import
    . (identifier) @import.path
    (identifier) @import.name)) @import

(using_statement
  (selected_import
    . (import_path) @import.path
    (identifier) @import.name)) @import

; --- selective with an aliased name: using Pkg: a as b ---------------------

(import_statement
  (selected_import
    . (identifier) @import.path
    (import_alias
      . (identifier) @import.name
      (identifier) @import.alias))) @import

(import_statement
  (selected_import
    . (import_path) @import.path
    (import_alias
      . (identifier) @import.name
      (identifier) @import.alias))) @import

(using_statement
  (selected_import
    . (identifier) @import.path
    (import_alias
      . (identifier) @import.name
      (identifier) @import.alias))) @import

(using_statement
  (selected_import
    . (import_path) @import.path
    (import_alias
      . (identifier) @import.name
      (identifier) @import.alias))) @import
