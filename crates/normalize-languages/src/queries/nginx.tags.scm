; Nginx tags query
; Covers: block directives (server, location, upstream, http, events, etc.)
; Nginx config is structured as block_directive nodes with a name: (directive) field.
; e.g., server { ... }, location /api { ... }, upstream backend { ... }

; Block directives: named blocks that form the top-level structure
(block_directive
  name: (directive) @name) @definition.module

; Lua block directives (OpenResty/lua-nginx-module): access_by_lua_block { ... },
; content_by_lua_block { ... }, etc. These are a third directive kind (sibling to
; simple_directive/block_directive in `conf`/`block`'s children) with no `name`
; field — the keyword is an anonymous literal token (verified via node-types.json
; and `normalize syntax ast`; see nginx.calls.scm for the full 7-variant list).
; @name captures the keyword token itself since there is no separate name node.
(lua_block_directive
  [
    "access_by_lua_block"
    "balancer_by_lua_block"
    "body_filter_by_lua_block"
    "content_by_lua_block"
    "header_filter_by_lua_block"
    "log_by_lua_block"
    "rewrite_by_lua_block"
  ] @name) @definition.module
