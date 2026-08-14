; VHDL complexity query
; @complexity — nodes that increase cyclomatic complexity
; @nesting — nodes that increase nesting depth
;
; Beyond sequential if/case/loop, VHDL has structural (elaboration-time)
; branching via `generate` statements, and conditional concurrent signal
; assignments (`x <= a when cond else b;` / `with sel select x <= ...;`).
; Both are genuine branch points and were previously entirely uncounted —
; verified present in this crate's own sample.vhd fixture (the
; `full <= '1' when count = DEPTH else '0';` line uses
; `conditional_waveforms`).

; Complexity nodes
(if_statement) @complexity
(case_statement) @complexity
(loop_statement) @complexity
(if_generate_statement) @complexity
(case_generate_statement) @complexity
(conditional_waveforms) @complexity
(selected_waveforms) @complexity

; Nesting nodes
(if_statement) @nesting
(case_statement) @nesting
(loop_statement) @nesting
(if_generate_statement) @nesting
(case_generate_statement) @nesting
(for_generate_statement) @nesting
(function_body) @nesting
(procedure_body) @nesting
