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

;; --- complexity.scm / cfg.scm: branch/match/exception/boolean forms -------
;; All list_lit-headed forms except `loop`, which parses as its own
;; dedicated `loop_macro` node (see commonlisp.complexity.scm's header).

(defun variant-if (x)
  (if (> x 0) 'pos 'neg))

(defun variant-when (x)
  (when (> x 0) x))

(defun variant-unless (x)
  (unless (> x 0) x))

(defun variant-cond (x)
  (cond ((> x 0) 'pos) ((< x 0) 'neg) (t 'zero)))

(defun variant-case (x)
  (case x (1 'one) (2 'two) (t 'other)))

(defun variant-ccase (x)
  (ccase x (1 'one) (2 'two)))

(defun variant-typecase (x)
  (typecase x (integer 'int) (string 'str)))

(defun variant-etypecase (x)
  (etypecase x (integer 'int) (string 'str)))

(defun variant-ctypecase (x)
  (ctypecase x (integer 'int) (string 'str)))

(defun variant-do (n)
  (do ((i 0 (+ i 1))) ((= i n)) (print i)))

(defun variant-dolist (lst)
  (dolist (i lst) (print i)))

(defun variant-dotimes (n)
  (dotimes (i n) (print i)))

;; `loop` — dedicated `loop_macro` grammar node, not a list_lit-headed
;; `sym_lit` form like every other construct above.
(defun variant-loop (n)
  (loop for i from 1 to n collect i))

(defun variant-handler-case ()
  (handler-case (foo) (error (e) (print e))))

(defun variant-handler-bind ()
  (handler-bind ((error (lambda (c) (print c)))) (foo)))

(defun variant-restart-case ()
  (restart-case (error "x") (use-value (v) v)))

(defun variant-unwind-protect ()
  (unwind-protect (foo) (bar)))

(defun variant-and-or (x y)
  (and x (or y (not x))))

;; --- complexity.scm: nesting via lambda/let/let*/flet/labels ---------------
(defun variant-nesting (x)
  (let* ((a (lambda (n) (* n n))))
    (flet ((sq (n) (* n n)))
      (labels ((rec (n) (if (= n 0) 1 (* n (rec (- n 1))))))
        (+ (a x) (sq x) (rec x))))))

;; --- NEGATIVE: must NOT match as tags definitions --------------------------

;; a plain function call is not a definition
(plain-defun 1)

;; a let-bound local is not a top-level def
(let ((local-not-a-def 1)) local-not-a-def)
