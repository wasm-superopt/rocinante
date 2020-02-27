;; P16(x, y) Compute max of two integers
;; o_1 = bvxor(x, y)
;; o_2 = bvneg(bvuge(x, y))
;; o_3 = bvand(o_1, o_2)
;; res := bvxor(o_3, y)

(module
  (func $p16 (export "p16") (param i32 i32) (result i32)
    (i32.xor
      (i32.and
        (i32.xor (local.get 0) (local.get 1))
        (i32.sub
          (i32.const 0)
          (i32.ge_s (local.get 0) (local.get 1))
        )
      )
      (local.get 1)
    )
  )
)
