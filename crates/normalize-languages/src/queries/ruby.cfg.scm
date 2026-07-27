; Ruby CFG query
; Captures control flow nodes for CFG construction.
; See normalize-cfg for the full capture vocabulary.
;
; Verified against arborium Ruby grammar via
;   normalize syntax ast <file> --compact --depth=-1
;   normalize syntax query <query> -p <file> --show-source
;
; Ruby uses expression-oriented constructs: if/unless are both used.
; next is the continue equivalent; raise is the throw equivalent but
; is NOT a dedicated node type — `raise "msg"` is an ordinary (call
; method: (identifier)) and bare `raise` (re-raise inside rescue) is a
; plain (identifier), both matched here via #eq? on the text, same
; technique as Perl's die()/Lua's error(). `case/in` pattern matching
; (Ruby 3+) uses a DIFFERENT outer node type "case_match" (not "case")
; with "in_clause" arms (not "in_pattern", which doesn't exist).
; `begin`/`rescue`/`ensure` have no "body" field at all — the guarded
; body is the unnamed first child, matched with a leading "." anchor
; so it doesn't also swallow the trailing rescue/ensure clauses.
; `for`'s iterable is field "value" (an (in ...) wrapper), not
; "pattern" (which is the loop variable, not the CFG-relevant part).

; ---------------------------------------------------------------------------
; if / unless (branch)
; ---------------------------------------------------------------------------

(if
  condition: (_) @cfg.branch.condition
  consequence: (_) @cfg.branch.then
  alternative: (_) @cfg.branch.else
) @cfg.branch

(if
  condition: (_) @cfg.branch.condition
  consequence: (_) @cfg.branch.then
  .
  ; no alternative
) @cfg.branch

(elsif
  condition: (_) @cfg.branch.condition
  consequence: (_) @cfg.branch.then
) @cfg.branch

; unless is an inverted if
(unless
  condition: (_) @cfg.branch.condition
  consequence: (_) @cfg.branch.then
  alternative: (_) @cfg.branch.else
) @cfg.branch

(unless
  condition: (_) @cfg.branch.condition
  consequence: (_) @cfg.branch.then
  .
) @cfg.branch

; ---------------------------------------------------------------------------
; case / when / in (match)
; ---------------------------------------------------------------------------

(case
  value: (_) @cfg.match.scrutinee
  (when) @cfg.match.arm
) @cfg.match

; case/in (pattern matching, Ruby 3+) — a distinct "case_match" node,
; not "case", with "in_clause" arms, not "in_pattern".
(case_match
  value: (_) @cfg.match.scrutinee
  clauses: (in_clause) @cfg.match.arm
) @cfg.match

; ---------------------------------------------------------------------------
; while / until (loop)
; ---------------------------------------------------------------------------

(while
  condition: (_) @cfg.loop.condition
  body: (_) @cfg.loop.body
) @cfg.loop

(until
  condition: (_) @cfg.loop.condition
  body: (_) @cfg.loop.body
) @cfg.loop

; ---------------------------------------------------------------------------
; for (loop over collection)
; ---------------------------------------------------------------------------

(for
  value: (_) @cfg.loop.condition
  body: (_) @cfg.loop.body
) @cfg.loop

; ---------------------------------------------------------------------------
; begin / rescue / ensure (exception handling)
; ---------------------------------------------------------------------------

(begin
  . (_) @cfg.try.body
) @cfg.try

(rescue
  exceptions: (exceptions (_) @cfg.try.catch.type)
) @cfg.try.catch

(rescue) @cfg.try.catch

(ensure) @cfg.try.finally

; ---------------------------------------------------------------------------
; Exits
; ---------------------------------------------------------------------------

(return) @cfg.exit.return

(break) @cfg.exit.break

(next) @cfg.exit.continue

(call
  method: (identifier) @_fn
  (#eq? @_fn "raise")
) @cfg.exit.throw

((identifier) @_id
  (#eq? @_id "raise")) @cfg.exit.throw
