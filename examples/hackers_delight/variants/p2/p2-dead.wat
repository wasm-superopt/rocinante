;; Test whether an unsigned integer is of the form 2^(n-1)
(module
  (func $p2 (export "p2") (param i32) (result i32) (local i32)
    (local.set 1 (local.get 0))
    (i32.and
      (i32.add (local.get 0) (i32.const 1))
      (local.get 0))
  )
)

;; Above is equivalent to
;; local.get 0
;; i32.const 1
;; i32.add
;; local.get 0
;; i32.and
;; end

;; local.get 0
;; i32.const -1
;; i32.sub
;; local.get 0
;; i32.and
;; local.get 0
;; i32.and
;; end

;; nop
;; nop
;; local.get 0
;; local.get 0
;; i32.const -1
;; i32.sub
;; i32.and
;; end

;; local.get 0
;; i32.const 1
;; i32.mul
;; local.get 0
;; i32.const 1
;; i32.add
;; i32.and
;; end
