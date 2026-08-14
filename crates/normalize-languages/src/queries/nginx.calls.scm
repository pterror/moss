; Nginx calls query
; @call — directive name (nginx directives are effectively function calls)
; @call.qualifier — not applicable (nginx has no method receiver concept)
;
; In nginx configs, directives like `proxy_pass`, `listen`, `server_name` etc.
; are effectively calls to built-in functions. Both simple directives
; (e.g. `proxy_pass http://backend;`) and block directives (e.g. `server { ... }`)
; have a directive name that acts as the "function" being called.

; Simple directive: proxy_pass http://backend;
(simple_directive
  name: (directive) @call)

; Block directive: server { ... }, location /api { ... }
(block_directive
  name: (directive) @call)

; Lua block directive (OpenResty/lua-nginx-module): access_by_lua_block { ... },
; content_by_lua_block { ... }, etc. Unlike simple_directive/block_directive, the
; grammar does not expose the directive keyword via a `name: (directive)` field —
; node-types.json shows lua_block_directive's only child field is the opaque
; `lua_block` body; the keyword is an anonymous literal token, one of the 7
; variants below (confirmed via node-types.json and `normalize syntax ast`).
(lua_block_directive
  [
    "access_by_lua_block"
    "balancer_by_lua_block"
    "body_filter_by_lua_block"
    "content_by_lua_block"
    "header_filter_by_lua_block"
    "log_by_lua_block"
    "rewrite_by_lua_block"
  ] @call)
