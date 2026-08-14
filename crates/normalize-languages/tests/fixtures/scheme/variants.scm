; Completeness matrix for scheme.{tags,calls,imports,complexity,cfg}.scm.
; One construct per keyword/form variant these queries key off of, plus a
; NEGATIVE section for constructs that must not match.
;
; Scheme's grammar (arborium-scheme 2.17.0) has NO fields at all on `list`
; — every form is `list` with a leading `symbol` naming it, so "variants"
; here means "every keyword the queries branch on by name", not
; node-type field variants in the node-types.json sense.

; --- tags.scm: function via (define (name args) body) ---
(define (fn-paren-form x) (* x x))

; --- tags.scm: function via (define name (lambda ...)) ---
(define fn-lambda-form (lambda (x) (* x x)))

; --- tags.scm: function via (define name (case-lambda ...)) ---
(define fn-case-lambda-form
  (case-lambda
    ((x) x)
    ((x y) (+ x y))))

; --- tags.scm: constant/variable via (define name value) ---
(define const-form 42)

; --- tags.scm: define-record-type ---
(define-record-type <pair-record>
  (make-pair-record a b)
  pair-record?
  (a pair-record-a)
  (b pair-record-b))

; --- tags.scm: define-syntax ---
(define-syntax my-when
  (syntax-rules ()
    ((_ c body ...) (if c (begin body ...) #f))))

; --- imports.scm: plain (import (library name)) ---
(import (scheme base))

; --- imports.scm: plain (require 'library) ---
; (kept as a comment — `require` is not R7RS-standard and this grammar's
; sample already exercises plain `import`; see negative section below for
; the require-form query behavior check instead)

; --- imports.scm: (import (only (library) name ...)) — unwrap to library ---
(import (only (scheme base) car cdr))

; --- imports.scm: (import (except (library) name ...)) — unwrap to library ---
(import (except (scheme base) car))

; --- imports.scm: (import (prefix (library) prefix:)) — unwrap to library ---
(import (prefix (scheme base) base:))

; --- imports.scm: (import (rename (library) (old new))) — unwrap to library ---
(import (rename (scheme base) (car first)))

; --- complexity.scm / cfg.scm: if (branch) ---
(define (variant-if x)
  (if (> x 0) 'pos 'neg))

; --- complexity.scm / cfg.scm: when (branch, no else) ---
(define (variant-when x)
  (when (> x 0) x))

; --- complexity.scm / cfg.scm: unless (branch, no else) ---
(define (variant-unless x)
  (unless (> x 0) x))

; --- complexity.scm / cfg.scm: cond (match) ---
(define (variant-cond x)
  (cond
    ((> x 0) 'pos)
    ((< x 0) 'neg)
    (else 'zero)))

; --- complexity.scm / cfg.scm: case (match) ---
(define (variant-case x)
  (case x
    ((1) 'one)
    ((2) 'two)
    (else 'other)))

; --- complexity.scm: case-lambda (match-like, as a value not a form here) ---
; (already exercised above via fn-case-lambda-form)

; --- complexity.scm / cfg.scm: do (loop) ---
(define (variant-do n)
  (do ((i 0 (+ i 1)) (acc 0 (+ acc i)))
      ((= i n) acc)))

; --- complexity.scm: for-each (loop) ---
(define (variant-for-each lst)
  (for-each display lst))

; --- cfg.scm: named let (loop) ---
(define (variant-named-let lst)
  (let loop ((remaining lst) (acc 0))
    (if (null? remaining)
        acc
        (loop (cdr remaining) (+ acc 1)))))

; --- complexity.scm / cfg.scm: guard (exception handling) ---
(define (variant-guard thunk)
  (guard (e (#t (display e) #f))
    (thunk)))

; --- complexity.scm: and / or (short-circuit boolean) ---
(define (variant-and-or x y)
  (and (> x 0) (or (> y 0) (= y 0))))

; --- complexity.scm: nesting via lambda / let / let* / letrec ---
(define (variant-nesting x)
  (let* ((a (lambda (n) (* n n)))
         (b (letrec ((f (lambda (n) (if (= n 0) 1 (* n (f (- n 1)))))))
              (f x))))
    (+ (a x) b)))

; ---------------------------------------------------------------------------
; NEGATIVE section — constructs that must NOT match certain captures
; ---------------------------------------------------------------------------

; calls.scm: special-form keywords must NOT be captured as @call, even
; though they are syntactically `(list . (symbol) ...)` just like a real
; call. Exercises: define, let, cond, if, else, import, define-record-type.
(define (negative-calls-example lst)
  (let ((x (car lst)))
    (cond
      ((null? x) 'empty)
      (else 'nonempty))))

; tags.scm: `(define name (lambda ...))` must produce ONLY
; @definition.function, never also @definition.constant on the same node.
(define negative-dual-tag-check (lambda (x) x))

; cfg.scm: plain (non-named) let must NOT be treated as a named-let loop.
(define (negative-plain-let x)
  (let ((y (* x 2)))
    y))
