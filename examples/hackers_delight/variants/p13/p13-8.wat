;; P13(x) Sign function
;; o_1 = bvshr(x, 31)
;; o_2 = bvneg(x)
;; o_3 = bvshr(o_2, 31)
;; res := bvor(o_1, o_3)

(module
  (func $p13 (export "p13") (param i32) (result i32)
    local.get 0
    i32.const 0
    i32.lt_s
    i32.eqz
    local.get 0
    i32.const 1
    nop
    i32.lt_s
    i32.sub
  )
)
