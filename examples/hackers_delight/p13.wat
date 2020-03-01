;; P13(x) Sign function
;; o_1 = bvshr(x, 31)
;; o_2 = bvneg(x)
;; o_3 = bvshr(o_2, 31)
;; res := bvor(o_1, o_3)

(module
  (func $p13 (export "p13") (param i32) (result i32)
    (i32.or
      (i32.shr_s (local.get 0) (i32.const 31))
      (i32.shr_u
        (i32.sub (i32.const 0) (local.get 0))
        (i32.const 31)
      )
    )
  )
)

;; local.get 0
;; i32.const 31
;; i32.shr_s
;; i32.const 0
;; local.get 0
;; i32.sub
;; i32.const 31
;; i32.shr_u
;; i32.or
