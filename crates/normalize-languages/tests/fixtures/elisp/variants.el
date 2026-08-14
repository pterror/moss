;;; variants.el --- Completeness matrix for Emacs Lisp queries  -*- lexical-binding: t -*-

;; Completeness matrix for elisp's tags/imports/complexity/cfg queries. One
;; small, commented construct per node-shape variant, plus a NEGATIVE
;; section of near-miss constructs that must NOT produce the capture they
;; superficially resemble. See crates/normalize-languages/src/queries/elisp.*.scm.
;;
;; The central fact this fixture exercises: the arborium elisp grammar has
;; a small, fixed built-in keyword set (if/cond/while/and/or/condition-case/
;; let/setq/catch/progn/save-excursion/lambda/defvar/defconst) whose forms
;; parse as a `special_form` node with the keyword as a literal anonymous
;; token — NOT `(list (symbol) ...)`. Every other named form (when, unless,
;; dolist, dotimes, cl-loop, pcase, cl-case, until, defcustom,
;; condition-case-unless-debug, ignore-errors, ...) is *not* in that set
;; and stays an ordinary `list` with a `(symbol)` head. Verified per-keyword
;; via `normalize syntax ast`.

;; --- imports -----------------------------------------------------------

(require 'cl-lib)                    ; list-headed require
(load "helper.el")                   ; list-headed load
(load-theme 'modus-vivendi)          ; list-headed load-theme (activation)
(require-theme 'modus-themes)        ; list-headed require-theme (Emacs 29+ dependency form)

;; --- tags: definitions ---------------------------------------------------

(defvar variants-var 1)              ; special_form-headed defvar
(defconst variants-const 2)          ; special_form-headed defconst
(defcustom variants-custom 3         ; list-headed defcustom
  "A customizable variable."
  :type 'integer
  :group 'variants)

(defun variants-fn (x) x)            ; function_definition (dedicated node kind)
(defmacro variants-macro (x) `(list ,x)) ; macro_definition (dedicated node kind)

(cl-defstruct variants-point         ; list-headed cl-defstruct
  (x 0) (y 0))

(defclass variants-class ()          ; list-headed defclass (EIEIO)
  ((slot :initform nil)))

;; --- complexity / cfg: special_form-headed vs list-headed branch/loop ----

(defun variants-if (n)
  (if (> n 0) "positive" "non-positive")) ; special_form-headed if

(defun variants-cond (n)
  (cond ((= n 0) "zero") (t "other")))    ; special_form-headed cond

(defun variants-while (n)
  (while (> n 0) (setq n (1- n))))        ; special_form-headed while

(defun variants-and-or (a b)
  (and a (or b nil)))                     ; special_form-headed and/or

(defun variants-condition-case ()
  (condition-case e (error "x") (error e))) ; special_form-headed condition-case

(defun variants-when-unless (a b)
  (when a (unless b "only-a")))           ; list-headed when/unless

(defun variants-dolist-dotimes (lst n)
  (dolist (x lst) x)                      ; list-headed dolist
  (dotimes (i n) i))                      ; list-headed dotimes

(defun variants-until (n)
  (until (<= n 0) (setq n (1- n))))       ; list-headed until (cl-lib)

(defun variants-case-pcase (n)
  (case n (0 "zero") (t "other"))         ; list-headed case (cl-lib)
  (pcase n (0 "zero") (_ "other")))       ; list-headed pcase

(defun variants-cl-loop (lst)
  (cl-loop for x in lst collect x))       ; list-headed cl-loop

(defun variants-ignore-errors ()
  (ignore-errors (error "boom")))         ; list-headed ignore-errors

;; --- NEGATIVE: constructs that must NOT match ---------------------------

(defun variants-zero-branch-body (a b)
  "A function with NO branches — used to prove ordinary calls/data lists
no longer count as @complexity/@nesting (the original bug: a bare
`(list) @complexity` counted every parenthesized expression)."
  (+ a b))

(defun variants-negative-forms ()
  "setq/let/condition-case's exception var/and-or operands must not be
mistaken for @definition.constant — only defvar/defconst/defcustom name
the thing being defined."
  (let ((total 0))
    (setq total (+ total 1))
    (condition-case caught-var
        (error "x")
      (error caught-var))
    (and 'not-a-definition (or 'also-not-a-definition nil))
    total))
