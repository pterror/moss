; SQL calls query
; @call — function invocation
; @call.qualifier — not applicable
;
; SQL function calls are represented as `invocation` nodes. An `invocation`
; contains an `object_reference` (the function name, which has a `name` field
; with an `identifier`) followed by a parenthesized argument list. Window
; functions (`ROW_NUMBER() OVER (...)`) also wrap an `invocation` for the
; function itself, so no separate handling is needed for them.
;
; NOTE (bug fix): the `object_reference` must be anchored to the FIRST child
; of `invocation` — `invocation` also has a `unit` field that allows
; `object_reference`, used by `EXTRACT(field FROM source)` for the date-part
; keyword (e.g. `YEAR` in `EXTRACT(YEAR FROM ordered_at)`). An unanchored
; pattern spuriously matches this `unit` value as a second, bogus call
; (confirmed via real parse: `EXTRACT(YEAR FROM ordered_at)` produced both
; `EXTRACT` and `YEAR` as @call captures).

; Function call: func(args...) — e.g. COUNT(*), COALESCE(a, b), NOW()
(invocation
  . (object_reference
      name: (identifier) @call))
