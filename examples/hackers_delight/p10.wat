;; P10(x, y) Test if nlz(x) == nlz(y)
;; where nlz is number of leading zeros.
;; o_1 = bvand(x, y)
;; o_2 = bvxor(x, y)
;; res := bvule(o_1, o_2)

(module
  (func $p10 (export "p10") (param i32 i32) (result i32)
    (i32.le_u 
        (i32.xor (local.get 0) (local.get 1))
        (i32.and (local.get 0) (local.get 1))
    )
  )
)
