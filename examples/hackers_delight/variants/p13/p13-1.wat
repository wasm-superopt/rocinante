;; P13(x) Sign function
;; o_1 = bvshr(x, 31)
;; o_2 = bvneg(x)
;; o_3 = bvshr(o_2, 31)
;; res := bvor(o_1, o_3)

(module
  (func $p13 (export "p13") (param i32) (result i32)
    i32.const 1
    local.get 0
    nop
    i32.le_s
    local.get 0
    i32.clz
    nop
    i32.eqz
    i32.sub
  )
)
