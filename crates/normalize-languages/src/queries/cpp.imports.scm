; C++ imports query
; @import       — the entire #include or import directive (for line number)
; @import.path  — the header file path (quotes/angle-brackets stripped by Rust)

; #include "local_header.h"
(preproc_include
  path: (string_literal) @import.path) @import

; #include <system_header.h>
(preproc_include
  path: (system_lib_string) @import.path) @import

; `using` declarations/directives/aliases previously had zero imports
; coverage at all — only #include was tracked, despite `using` being the
; other primary way a C++ translation unit declares a dependency on another
; namespace/type. Precedented by c-sharp.imports.scm's `using_directive`
; handling of the analogous C# construct.

; using namespace std;  (using_declaration wrapping a bare identifier after
; the literal "namespace" keyword)
(using_declaration
  "namespace"
  (identifier) @import.path) @import

; using std::vector;  (using_declaration wrapping a qualified_identifier,
; no "namespace" keyword — brings one specific name into scope)
(using_declaration
  (qualified_identifier) @import.path) @import

; using IntVec = std::vector<int>;  (type alias declaration)
(alias_declaration
  name: (type_identifier) @import.alias
  type: (type_descriptor) @import.path) @import

; namespace fs = std::filesystem;  (namespace alias, nested path)
(namespace_alias_definition
  name: (namespace_identifier) @import.alias
  (nested_namespace_specifier) @import.path) @import

; namespace short_name = long_name;  (namespace alias, single-segment path)
(namespace_alias_definition
  name: (namespace_identifier) @import.alias
  (namespace_identifier) @import.path) @import
