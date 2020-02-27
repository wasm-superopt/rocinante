;; P19(x, m, k): Exchanging 2 fields A and B of the same register x where m is
;; mask which identifies field B and k is number of bits from end of A to start
;; of B
;; o_1 = bvshr(x, k)
;; o_2 = bvxor(x, o_1)
;; o_3 = bvand(o_2, m)
;; o_4 = bvshl(o_3, k)
;; o_5 = bvxor(o_4, o_3)
;; res = bvxor(o_5, x)

(module
  (func $p19 (export "p19") (param i32 i32 i32) (result i32) (local i32)
    (local.set 3
      (i32.and ;; o_3
        (i32.xor ;; o_2
          (local.get 0)
          (i32.shr_u (local.get 0) (local.get 2)) ;; o_1
        )
        (local.get 1)
      )
    )
    (i32.xor
      (i32.xor ;; o_5 
        (i32.shl ;; o_4
          (local.get 3)
          (local.get 2)
        )
        (local.get 3)
      )
      (local.get 0)
    )
  )
)
