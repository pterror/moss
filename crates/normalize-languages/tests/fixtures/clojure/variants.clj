; Completeness-matrix fixture for Clojure query files.
; One small, commented construct per node-type / field-variant found by
; cross-referencing arborium-clojure 2.17.0's node-types.json against
; clojure.{tags,imports,calls}.scm, plus a NEGATIVE section of near-miss
; constructs that must not match.

; --- tags.scm: definition forms -------------------------------------------

; defn — @definition.function
(defn plain-fn [x] x)

; defn- — private function, @definition.function (Visibility::Private via
; the trailing '-' convention)
(defn- private-fn [x] x)

; defn with ^:private reader metadata on the name — the OTHER private-def
; convention (at least as common as defn- in real Clojure). @name must be
; the bare symbol "meta-private-fn", not "^:private meta-private-fn".
(defn ^:private meta-private-fn [x] x)

; defn with a docstring before the multi-arity bodies
(defn multi-arity-fn
  "doc"
  ([x] x)
  ([x y] (+ x y)))

; defmacro — @definition.macro
(defmacro my-macro [x] `(println ~x))

; defmethod — @definition.method (dispatches on a defmulti below)
(defmulti shape-area :shape)
(defmethod shape-area :circle [c] (:r c))

; ns — @definition.module
(ns variants.ns-form
  (:require [clojure.string :as str]))

; defrecord — @definition.class
(defrecord VRecord [a b])

; deftype — @definition.class
(deftype VType [a b])

; defprotocol — @definition.interface
(defprotocol VProto
  (proto-fn [this]))

; definterface — @definition.interface (Java-interop sibling of
; defprotocol; same (list_lit . sym_lit=form . sym_lit=name) shape)
(definterface VInterface
  (iface-fn [this]))

; def — @definition.constant
(def a-constant 42)

; --- imports.scm: require/import shapes ------------------------------------

; bare top-level quoted require vector with :as alias — the common
; outside-of-ns idiom; parses as quoting_lit wrapping vec_lit, NOT a bare
; vec_lit child of list_lit.
(require '[variants.aliased :as va])

; multiple quoted require vectors in one call
(require '[variants.multi-a :as vma] '[variants.multi-b :as vmb])

; bare quoted require vector, no alias
(require '[variants.no-alias])

; import — package-grouped Java classes: @import.path = pkg, @import.name =
; each class
(import (java.util UUID Random))

; import — single quoted fully-qualified class
(import 'java.util.Date)

; import — single bare fully-qualified class
(import java.io.File)

; ns form with :import clause, both package-grouped and bare-class shapes
(ns variants.ns-with-import
  (:require [variants.req-in-ns :as vrin])
  (:import (java.util List Map)
           java.io.InputStream))

; --- NEGATIVE: must NOT match --------------------------------------------

; :refer names inside a require vector must never be captured as
; @import.path/@import.name — they name what's pulled INTO scope, not a
; namespace/class being imported.
(require '[variants.refer-ns :refer [refer-a refer-b]])

; a plain function call is not a definition
(plain-fn 1)

; a let-bound local is not a top-level def
(let [local-not-a-def 1] local-not-a-def)

; an anonymous fn literal has no name to capture
(fn [x] x)
#(inc %)
