; Nginx complexity query
; @complexity — nodes that increase cyclomatic complexity
; @nesting — nodes that increase nesting depth

; Complexity nodes (each block directive adds nesting/branching)
(block_directive) @complexity

; Lua block directives (access_by_lua_block { ... }, content_by_lua_block { ... }, etc.)
; also introduce a block scope, so they count the same as block_directive. The lua_code
; body itself is opaque to this grammar (it is not a Lua parse tree, just raw text) —
; we cannot analyze complexity *inside* the embedded Lua, only the fact that a block exists.
(lua_block_directive) @complexity

; Nesting nodes
(block_directive) @nesting
(lua_block_directive) @nesting
