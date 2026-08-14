; R imports query
; @import       — the entire library/require call (for line number)
; @import.path  — the package name being imported
;
; `arguments`'s children include a `comma` node type that (per node-types.json)
; is *named* (unusual — most grammars treat separators as anonymous), so an
; unanchored `(_) @import.path` inside `arguments` matches every child: the
; package-name argument, any trailing named argument (e.g.
; `character.only = TRUE`), and the comma tokens themselves. Anchoring to the
; first child (`.`) and capturing `argument.value` (not the whole `argument`
; node, which for a named argument like `library(package = "dplyr")` would
; include the `package = ` prefix) fixes both problems: only the first,
; positional argument is ever captured, and only its value text.

; library(pkg) or library("pkg") or library(package = "pkg")
(call
  function: (identifier) @_f (#eq? @_f "library")
  arguments: (arguments
    .
    (argument value: (_) @import.path))) @import

; require(pkg)
(call
  function: (identifier) @_f (#eq? @_f "require")
  arguments: (arguments
    .
    (argument value: (_) @import.path))) @import

; requireNamespace("pkg") — conditional/soft-dependency check, still a
; genuine package reference.
(call
  function: (identifier) @_f (#eq? @_f "requireNamespace")
  arguments: (arguments
    .
    (argument value: (_) @import.path))) @import
