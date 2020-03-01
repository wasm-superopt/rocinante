;; P13(x) Sign function
;; o_1 = bvshr(x, 31)
;; o_2 = bvneg(x)
;; o_3 = bvshr(o_2, 31)
;; res := bvor(o_1, o_3)

(module
  (func $p13 (export "p13") (param i32) (result i32)
    local.get 0
    local.get 0
    i32.ctz
    i32.shr_s
    i32.const 2
    local.get 0
    i32.eqz
    i32.rotl
    i32.rem_s
  )
)
