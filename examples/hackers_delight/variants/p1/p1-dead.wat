;; Turn-off rightmost 1 bit.
(module
  (func $p1 (export "p1") (param i32) (result i32)
    (nop)
    (i32.and
      (i32.sub (local.get 0) (i32.const 1))
      (local.get 0))
    (nop)
  )
)

;; Above is equivalent to
;; nop
;; local.get 0
;; i32.const 1
;; i32.sub
;; local.get 0
;; i32.and
;; nop
;; end

;; local.get 0
;; i32.const -1
;; local.get 0
;; i32.add
;; nop
;; nop
;; i32.and
;; end

;; local.get 0
;; nop
;; i32.const 1
;; i32.sub
;; local.get 0
;; i32.and
;; nop
;; end
