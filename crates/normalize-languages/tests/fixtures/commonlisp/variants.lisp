;; Completeness-matrix fixture for Common Lisp query files.
;; One small, commented construct per node-type / field-variant found by
;; cross-referencing arborium-commonlisp 2.17.0's node-types.json against
;; commonlisp.{tags,imports,calls}.scm, plus a NEGATIVE section of near-miss
;; constructs that must not match.

;; --- tags.scm: the unified `defun` grammar node --------------------------
;;
;; defun/defgeneric/defmethod/defmacro all parse as ONE `defun` grammar
;; node discriminated only by `defun_header.keyword`'s text — verified via
;; `normalize syntax query`. The kind (@definition.function/.method/.macro)
;; must track that keyword, not assume structural shape implies kind.

;; defun — @definition.function, function_name: (sym_lit)
(defun plain-defun (x) x)

;; defun with a setf-expander name — function_name: (list_lit), a
;; previously-uncaptured shape (@name must be "(setf setter-target)", not
;; empty/dropped).
(defun (setf setter-target) (new-value obj)
  (setf (slot-value obj 'x) new-value))

;; defgeneric — @definition.function
(defgeneric generic-op (x))

;; defmethod — @definition.method (previously misclassified as
;; @definition.function because the dedicated list_lit-based defmethod
;; pattern never matched this grammar's actual defmethod shape)
(defmethod generic-op ((x integer))
  (* x 2))

;; defmethod with a qualifier (:before/:after/:around) — qualifier must not
;; interfere with name extraction
(defmethod generic-op :before ((x integer))
  (print "before"))

;; defmethod with a setf-expander name
(defmethod (setf setter-target) (new-value obj)
  (setf (slot-value obj 'y) new-value))

;; defmacro — @definition.macro (previously misclassified as
;; @definition.function for the same reason as defmethod above)
(defmacro plain-defmacro (x)
  `(+ ,x 1))

;; --- tags.scm: plain list_lit forms (unchanged shape) ----------------------

(defclass plain-class () ())

(defstruct plain-struct x y)

(defpackage :plain-package
  (:use :cl))

(deftype plain-type () 'integer)

(defconstant +plain-constant+ 42)

(defparameter *plain-parameter* 0)

;; --- imports.scm: require/use-package/ql:quickload ------------------------

;; require with a quoted symbol
(require 'variants-quoted-dep)

;; require with a string
(require "variants-string-dep")

;; use-package with a keyword
(use-package :variants-used-pkg)

;; ql:quickload — package-qualified symbol parses as `package_lit`, not
;; `sym_lit`; previously entirely unmatched.
(ql:quickload :variants-quickloaded-pkg)

;; --- calls.scm: special forms must NOT be calls; real calls must be -------

(defun calls-negative-check (n)
  ;; a real function call
  (format t "~a" n)
  ;; special forms/macros sharing (leading-sym args...) syntax with calls
  (if (> n 0)
      (let ((y (* n 2)))
        (dolist (i (list 1 2 3))
          (when (> i 0)
            (print i)))
        y)
      0))

;; --- NEGATIVE: must NOT match as tags definitions --------------------------

;; a plain function call is not a definition
(plain-defun 1)

;; a let-bound local is not a top-level def
(let ((local-not-a-def 1)) local-not-a-def)
