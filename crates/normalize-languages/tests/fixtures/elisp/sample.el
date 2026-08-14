;;; sample.el --- Sample Emacs Lisp file  -*- lexical-binding: t -*-

(require 'cl-lib)
(require 'subr-x)

;;; Variables

(defvar sample-counter 0
  "A simple counter variable.")

(defconst sample-max-value 100
  "Maximum allowed value.")

(defcustom sample-greeting "Hello"
  "Greeting string used by sample functions."
  :type 'string
  :group 'sample)

;;; Functions

(defun sample-greet (name)
  "Greet NAME with the configured greeting."
  (message "%s, %s!" sample-greeting name))

(defun sample-factorial (n)
  "Return the factorial of N."
  (if (<= n 1)
      1
    (* n (sample-factorial (- n 1)))))

(defun sample-sum-list (lst)
  "Return the sum of all elements in LST."
  (let ((total 0))
    (dolist (item lst)
      (setq total (+ total item)))
    total))

(defun sample-filter (pred lst)
  "Return elements of LST satisfying PRED."
  (let ((result '()))
    (dolist (item lst)
      (when (funcall pred item)
        (push item result)))
    (nreverse result)))

;;; Struct

(cl-defstruct sample-point
  "A 2D point."
  (x 0 :type number)
  (y 0 :type number))

(defun sample-distance (p1 p2)
  "Return Euclidean distance between P1 and P2."
  (let ((dx (- (sample-point-x p2) (sample-point-x p1)))
        (dy (- (sample-point-y p2) (sample-point-y p1))))
    (sqrt (+ (* dx dx) (* dy dy)))))

;;; Control flow idioms

(defun sample-classify (n)
  "Classify N as negative, zero, or positive."
  (cond
   ((< n 0) "negative")
   ((= n 0) "zero")
   (t "positive")))

(defun sample-wait-until-ready (get-status)
  "Loop with WHILE until GET-STATUS returns non-nil."
  (while (not (funcall get-status))
    (sit-for 0.1))
  t)

(defun sample-safe-divide (a b)
  "Divide A by B, returning nil on error via CONDITION-CASE."
  (condition-case err
      (/ a b)
    (arith-error (message "division error: %s" err) nil)))

(defun sample-both-set (a b)
  "Return non-nil only when both A and B are set (AND/OR short-circuit)."
  (and a (or b nil)))

(defun sample-make-adder (n)
  "Return a closure (LAMBDA) that adds N to its argument."
  (lambda (x) (+ x n)))

(provide 'sample)
;;; sample.el ends here
