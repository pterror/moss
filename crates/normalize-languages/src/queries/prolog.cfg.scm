; Prolog CFG query
; Captures control flow nodes for CFG construction.
; See normalize-cfg for the full capture vocabulary.
;
; Verified against arborium Prolog grammar via
;   normalize syntax ast <file> --compact --depth=-1
;   normalize syntax query <query> -p <file> --show-source
; Note: the CLI's language registry resolves ".pl" to Prolog, but Perl
; also claims ".pl" — probing was done through ".prolog" fixture
; copies to force Prolog resolution unambiguously (documented as a
; pre-existing collision in TODO.md, not fixed here).
;
; This grammar is a generic term reader — there is no "if_then",
; "if_then_else", or "call" node type at all. Everything reduces to
; (operator_notation left operator: (X) right) where X is either a
; dedicated node type for a few operators (";" is (semicolon)) or the
; generic (binary_operator) carrying the operator text (e.g. "->",
; ":-", "is"), matched here via #eq? on that text. Predicate calls like
; catch(...)/throw(...) are (functional_notation function: (atom) ...)
; — an ordinary compound term, indistinguishable from any other call
; except by matching the functor name via #eq?, same technique.
;
; Prolog's control flow is via clause matching, cut (!), and
; meta-predicates (-> for if-then, ; for disjunction/if-then-else,
; catch/3). There are no loops/break/continue in the imperative sense
; — recursion replaces iteration, so no cfg.loop captures exist here.

; ---------------------------------------------------------------------------
; if-then-else — (Cond -> Then ; Else), modeled as
; (Cond -> Then) ; Else nested one level via the semicolon operator
; ---------------------------------------------------------------------------

(operator_notation
  (operator_notation
    (_) @cfg.branch.condition
    operator: (binary_operator) @_op
    (_) @cfg.branch.then
    (#eq? @_op "->")
  )
  operator: (semicolon)
  (_) @cfg.branch.else
) @cfg.branch

; ---------------------------------------------------------------------------
; if-then (bare Cond -> Then, no disjunction/else)
; ---------------------------------------------------------------------------

(operator_notation
  (_) @cfg.branch.condition
  operator: (binary_operator) @_op
  (_) @cfg.branch.then
  (#eq? @_op "->")
) @cfg.branch

; ---------------------------------------------------------------------------
; catch/3 (exception handling: catch(Goal, Catcher, Recovery))
; ---------------------------------------------------------------------------

(functional_notation
  function: (atom) @_fn
  (arg_list
    (_) @cfg.try.body
    (arg_list_separator)
    (_) @cfg.try.catch
    (arg_list_separator)
    (_)
  )
  (#eq? @_fn "catch")
) @cfg.try

; ---------------------------------------------------------------------------
; Exits
; ---------------------------------------------------------------------------

; throw/1
(functional_notation
  function: (atom) @_fn
  (arg_list (_))
  (#eq? @_fn "throw")
) @cfg.exit.throw
