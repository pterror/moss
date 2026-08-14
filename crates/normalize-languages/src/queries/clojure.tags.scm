; Clojure tags query
;
; Clojure is a Lisp — all forms are list_lit nodes.
; Definitions use leading sym_lit: defn, defmacro, ns, defrecord, defprotocol, def.
;
; @name always captures `sym_lit`'s `name:` field (a `sym_name` node), not the
; whole `sym_lit`. `sym_lit` also has an optional `meta:` field (reader
; metadata like `^:private`/`^{:doc "..."}` prefixed on the symbol) which
; extends the `sym_lit` node's own span to include the metadata text —
; verified via `normalize syntax ast` on `(defn ^:private foo ...)`, where
; the whole `sym_lit` node's text is `^:private foo`. Capturing the bare
; `sym_lit` (as this file did before) leaked that metadata prefix into the
; extracted symbol name; capturing `name:` gives the clean identifier.

; (defn name [...] ...)
(list_lit
  (sym_lit) @_kw (#eq? @_kw "defn")
  .
  (sym_lit name: (sym_name) @name)) @definition.function

; (defn- name [...] ...) — private function
(list_lit
  (sym_lit) @_kw (#eq? @_kw "defn-")
  .
  (sym_lit name: (sym_name) @name)) @definition.function

; (defmacro name [...] ...)
(list_lit
  (sym_lit) @_kw (#eq? @_kw "defmacro")
  .
  (sym_lit name: (sym_name) @name)) @definition.macro

; (defmethod name ...)
(list_lit
  (sym_lit) @_kw (#eq? @_kw "defmethod")
  .
  (sym_lit name: (sym_name) @name)) @definition.method

; (defmulti name ...) — declares the generic dispatch function that
; (defmethod ...) forms attach implementations to; symmetric with defmethod
; above and just as common in real Clojure (multimethod dispatch idiom).
(list_lit
  (sym_lit) @_kw (#eq? @_kw "defmulti")
  .
  (sym_lit name: (sym_name) @name)) @definition.function

; (ns name ...)
(list_lit
  (sym_lit) @_kw (#eq? @_kw "ns")
  .
  (sym_lit name: (sym_name) @name)) @definition.module

; (defrecord Name [...] ...)
(list_lit
  (sym_lit) @_kw (#eq? @_kw "defrecord")
  .
  (sym_lit name: (sym_name) @name)) @definition.class

; (deftype Name [...] ...)
(list_lit
  (sym_lit) @_kw (#eq? @_kw "deftype")
  .
  (sym_lit name: (sym_name) @name)) @definition.class

; (defprotocol Name ...)
(list_lit
  (sym_lit) @_kw (#eq? @_kw "defprotocol")
  .
  (sym_lit name: (sym_name) @name)) @definition.interface

; (definterface Name ...) — Java-interop sibling of defprotocol; same shape.
(list_lit
  (sym_lit) @_kw (#eq? @_kw "definterface")
  .
  (sym_lit name: (sym_name) @name)) @definition.interface

; (def name ...)
(list_lit
  (sym_lit) @_kw (#eq? @_kw "def")
  .
  (sym_lit name: (sym_name) @name)) @definition.constant
