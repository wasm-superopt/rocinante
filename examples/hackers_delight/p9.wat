;; P9(x) Absolute value function
;; o_1 = bvshr(x, 31)
;; o_2 = bvxor(x, o_1)
;; res := bvsub(o_2, o_1)

;; An interesting example as it uses a local, and a constant other than 
;; -2, -1, 0, 1, -2.

(module
  (func $p9 (export "p9") (param i32) (result i32) (local i32)
    (local.set 1 (i32.shr_s (local.get 0) (i32.const 31)))
    (i32.sub
        (i32.xor (local.get 0) (local.get 1))
        (local.get 1)
    )
  )
)
