;; P13(x) Sign function
;; o_1 = bvshr(x, 31)
;; o_2 = bvneg(x)
;; o_3 = bvshr(o_2, 31)
;; res := bvor(o_1, o_3)

(module
  (func $p13 (export "p13") (param i32) (result i32)
    local.get 0
    local.get 0
    i32.eqz
    i32.ge_s
    i32.const 0
    local.get 0
    i32.clz
    i32.eq
    i32.sub
  )
)
