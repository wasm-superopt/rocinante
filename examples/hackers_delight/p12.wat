;; P12(x, y) Test if nlz(x) <= nlz(y)
;; where nlz is number of leading zeros.
;; o_1 = bvnot(x)
;; o_2 = bvand(y, o_1)
;; res := bvule(o_2, x)

(module
  (func $p12 (export "p12") (param i32 i32) (result i32)
    (i32.le_u
      (i32.and
        (local.get 1)
        (i32.xor (local.get 0) (i32.const -1))
      )
      (local.get 0)
    )
  )
)

;; local.get 1
;; local.get 0
;; i32.const -1
;; i32.xor
;; i32.and
;; local.get 0
;; i32.le_u
