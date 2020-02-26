;; P15(x, y) Ceil of average of two integers without over-flowing
;; o_1 = bvor(x, y)
;; o_2 = bvxor(x, y)
;; o_3 = bvshr(o_2, 1)
;; res := bvadd(o_1, o_3)

(module
  (func $p15 (export "p15") (param i32 i32) (result i32)
    (i32.sub
      (i32.or (local.get 0) (local.get 1))
      (i32.shr_s
        (i32.xor (local.get 0) (local.get 1))
        (i32.const 1)
      )
    )
  )
)
