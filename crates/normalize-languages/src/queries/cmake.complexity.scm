; CMake complexity query
; @complexity — nodes that increase cyclomatic complexity
; @nesting — nodes that increase nesting depth

; Complexity nodes
(if_condition) @complexity
(elseif_command) @complexity
(foreach_loop) @complexity
(while_loop) @complexity

; Nesting nodes
(if_condition) @nesting
(foreach_loop) @nesting
(while_loop) @nesting

; Scope-introducing constructs also count as nesting, matching the
; established cross-language convention (e.g. rust.complexity.scm counts
; function_item/impl_item/mod_item, python.complexity.scm counts
; function_definition/class_definition) — a function/macro/block body reads
; one level deeper than its surrounding scope even though it isn't itself a
; decision point. `block_def` (CMake's `block()...endblock()` scope-isolation
; construct) was entirely unhandled by every cmake.*.scm file before this;
; confirmed present in the grammar via `normalize syntax query`.
(function_def) @nesting
(macro_def) @nesting
(block_def) @nesting
