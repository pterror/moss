; HTML imports query
; @import       — the containing element (for line number)
; @import.path  — the URL/path being loaded
;
; Tag/attribute names are matched case-insensitively (#match? "(?i)...") since
; HTML tag and attribute names are case-insensitive per spec, and this grammar
; preserves source casing verbatim in `tag_name`/`attribute_name` text (verified
; via `normalize syntax query` against `<LINK HREF=...>`, `<IMG SRC=...>`).
;
; Attribute values come in two node kinds depending on whether the source
; quotes them: `quoted_attribute_value` (`href="x.css"`) or `attribute_value`
; (`href=x.css`, legal unquoted HTML). Both are handled for every reference
; attribute below — the query only handled the quoted form previously, which
; silently dropped every unquoted src/href.

; <script src="app.js"></script> / <script src=app.js></script>
(script_element
  (start_tag
    (attribute
      (attribute_name) @_attr
      [(quoted_attribute_value) (attribute_value)] @import.path)
    (#match? @_attr "(?i)^src$"))) @import

; <link href="styles.css"> (void element, implicit close)
(element
  (start_tag
    (tag_name) @_tag
    (attribute
      (attribute_name) @_attr
      [(quoted_attribute_value) (attribute_value)] @import.path)
    (#match? @_tag "(?i)^link$")
    (#match? @_attr "(?i)^href$"))) @import

; <link href="styles.css" /> (self-closing syntax)
(element
  (self_closing_tag
    (tag_name) @_tag
    (attribute
      (attribute_name) @_attr
      [(quoted_attribute_value) (attribute_value)] @import.path)
    (#match? @_tag "(?i)^link$")
    (#match? @_attr "(?i)^href$"))) @import

; <img src="pic.png"> (void element, implicit close)
(element
  (start_tag
    (tag_name) @_tag
    (attribute
      (attribute_name) @_attr
      [(quoted_attribute_value) (attribute_value)] @import.path)
    (#match? @_tag "(?i)^img$")
    (#match? @_attr "(?i)^src$"))) @import

; <img src="pic.png" /> (self-closing syntax)
(element
  (self_closing_tag
    (tag_name) @_tag
    (attribute
      (attribute_name) @_attr
      [(quoted_attribute_value) (attribute_value)] @import.path)
    (#match? @_tag "(?i)^img$")
    (#match? @_attr "(?i)^src$"))) @import
