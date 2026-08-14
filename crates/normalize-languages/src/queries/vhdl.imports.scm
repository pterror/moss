; VHDL imports query
; @import       — the entire use clause / context reference (for line number)
; @import.path  — the library/package path
; @import.glob  — .all wildcard marker

; use ieee.std_logic_1164.all;
(use_clause) @import.path @import

; VHDL-2008 context reference: context work.ieee_ctx;
; (pulls in a whole named `context` declaration's clauses — analogous to a
; grouped `use`. Verified via `normalize syntax query` against a probe file;
; see crates/normalize-languages/tests/query_fixtures.rs
; vhdl_imports_completeness. `library_clause` is deliberately still excluded
; — it only declares a logical library name visible, with no member path,
; unlike `use_clause`/`context_reference` which name a concrete unit.)
(context_reference) @import.path @import
