; DeviceTree imports query
; @import       — the entire #include directive (for line number)
; @import.path  — the header/overlay file path

; #include "board.dtsi"
(preproc_include
  path: (string_literal) @import.path) @import

; #include <dt-bindings/gpio/gpio.h>
(preproc_include
  path: (system_lib_string) @import.path) @import

; #include SOC_DTS_HEADER  (macro-expanded include target; `path` field
; allows a bare `identifier` per node-types.json in addition to the two
; string forms above — the actual filename isn't known statically since
; it depends on preprocessor macro expansion, but the directive itself is
; still worth reporting as an import edge)
(preproc_include
  path: (identifier) @import.path) @import
