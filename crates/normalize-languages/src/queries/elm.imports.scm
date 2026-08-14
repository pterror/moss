; Elm imports query
; @import       — the entire import clause (for line number)
; @import.path  — the module name
; @import.alias — the "as Alias" name
; @import.glob  — exposing (..) wildcard marker
; @import.name  — a single exposed name

; import Html
(import_clause
  moduleName: (upper_case_qid) @import.path) @import

; import Html as H
(import_clause
  moduleName: (upper_case_qid) @import.path
  asClause: (as_clause
    (upper_case_identifier) @import.alias)) @import

; import Html exposing (..)
(import_clause
  moduleName: (upper_case_qid) @import.path
  exposing: (exposing_list
    (double_dot) @import.glob)) @import

; import Html exposing (div, span)
(import_clause
  moduleName: (upper_case_qid) @import.path
  exposing: (exposing_list
    (exposed_value) @import.name)) @import

; import Json.Decode exposing (Decoder, decodeString)
; import Html exposing (Html(..))
; `exposing_list`'s children also allow `exposed_type` (exposing a type,
; optionally with its constructors via `Type(..)`) — verified via
; `normalize syntax query`: this is an extremely common real-world Elm
; idiom (any module using an imported type re-exposes it this way), and
; was entirely unmatched by the `exposed_value`-only pattern above.
(import_clause
  moduleName: (upper_case_qid) @import.path
  exposing: (exposing_list
    (exposed_type) @import.name)) @import

; import Basics exposing ((+), (-))
; `exposed_operator` — rare in modern Elm (0.19 removed user-defined
; operators) but still a real, parseable exposing-list entry.
(import_clause
  moduleName: (upper_case_qid) @import.path
  exposing: (exposing_list
    (exposed_operator
      operator: (_) @import.name))) @import
