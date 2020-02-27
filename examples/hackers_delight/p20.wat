;; P20(x): Next higher unsigned number with same number of 1 bits.
;; o_1 = bvneg(x)
;; o_2 = bvand(x, o_1)
;; o_3 = bvadd(x, o_3)
;; o_4 = bvxor(x, o_2)
;; o_5 = bvshr(o_4, 2)
;; o_6 = bvdiv(o_5, o_2)
;; res = bvor(o_6, o_3)

(module
  (func $p20 (export "p20") (param i32) (result i32) (local i32 i32)
    (local.set 1
      (i32.and ;; o_2
        (local.get 0)
        (i32.sub (i32.const 0) (local.get 0)) ;; o_1
      )
    )
    (local.set 2
      (i32.add (local.get 0) (local.get 1)) ;; o_3
    )
    (i32.or
      (i32.div_u ;; o_6
        (i32.shr_u ;; o_5
          (i32.xor ;; o_4
            (local.get 0)
            (local.get 2)
          )
          (i32.const 2)
        )
        (local.get 1)
      )
      (local.get 2)
    )
  )
)
