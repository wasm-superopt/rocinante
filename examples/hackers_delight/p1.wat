;; Turn-off rightmost 1 bit.
(module
  (func $p1 (export "p1") (param i32) (result i32)
    (i32.and
      (i32.sub (get_local 0) (i32.const 1))
      (get_local 0))
  )
)

;; Above is equivalent to
;; get_local 0
;; i32.const 1
;; i32.sub
;; get_local 0
;; i32.and
;; end
