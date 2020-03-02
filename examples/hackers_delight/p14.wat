;; P14(x, y) Floor of average of two integers without over-flowing
;; o_1 = bvand(x, y)
;; o_2 = bvxor(x, y)
;; o_3 = bvshr(o_2, 1)
;; res := bvadd(o_1, o_3)

(module
  (func $p14 (export "p14") (param i32 i32) (result i32)
    (i32.add
      (i32.and (local.get 0) (local.get 1))
      (i32.shr_s
        (i32.xor (local.get 0) (local.get 1))
        (i32.const 1)
      )
    )
  )
)

;; local.get 0
;; local.get 1
;; i32.and
;; local.get 0
;; local.get 1
;; i32.xor
;; i32.const 1
;; i32.shr_s
;; i32.add
