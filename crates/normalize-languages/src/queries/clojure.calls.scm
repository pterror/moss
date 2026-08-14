; Clojure calls query
; @call — call expression nodes
; @call.qualifier — qualifier/receiver for namespace-qualified calls
;
; Clojure is a Lisp: function application is `(f args...)` where f is
; the first element of a list. In the tree-sitter grammar, this is a
; `list_lit` with a leading `sym_lit` (symbol) as the function position.
;
; We match the outer list_lit as @call and the first sym_lit as @call (name).
; Namespace-qualified symbols like `ns/func` are `sym_lit` nodes containing
; a `/`.
;
; The syntax for a function call and a special-form/macro invocation is
; IDENTICAL in a Lisp — both are `(leading-symbol args...)`. Without
; excluding known special forms and definition/control-flow macros, every
; `(defn foo ...)`, `(let [...] ...)`, `(cond ...)`, `(when ...)` etc. in a
; file was counted as a "call" to `defn`/`let`/`cond`/`when` — confirmed via
; `normalize syntax query` against `tests/fixtures/clojure/sample.clj`, where
; unfiltered the query reported `defn` as a call at every one of the file's
; five function definitions. These aren't calls in any call-graph sense (you
; don't "call" `let`), so they're excluded below. List drawn from Clojure's
; documented special forms (https://clojure.org/reference/special_forms) plus
; the definition/control-flow core macros this codebase's own
; `clojure.tags.scm`/`clojure.cfg.scm` already treat as structural, not
; call, constructs.
(list_lit
  .
  (sym_lit) @call
  (#not-match?
    @call
    "^(def|defn|defn-|defmacro|defmethod|defmulti|defrecord|deftype|defprotocol|definterface|declare|ns|in-ns|import|require|use|let|let\\*|letfn|letfn\\*|loop|loop\\*|recur|if|if-let|if-not|when|when-not|when-let|when-first|cond|condp|case|case\\*|for|doseq|dotimes|while|try|catch|finally|throw|do|quote|var|fn|fn\\*|new|set!|monitor-enter|monitor-exit|->|->>|some->|some->>|as->|extend-protocol|extend-type|reify|proxy|comment)$"))

; Namespace-qualified call: (ns/func args...)
; The sym_lit itself serves as the call name; we treat the namespace portion
; as qualifier by convention (sym_lit text contains ns/func)
