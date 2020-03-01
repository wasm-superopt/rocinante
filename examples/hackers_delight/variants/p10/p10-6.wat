;; P10(x, y) Test if nlz(x) == nlz(y)
;; where nlz is number of leading zeros.
;; o_1 = bvand(x, y)
;; o_2 = bvxor(x, y)
;; res := bvule(o_1, o_2)

(module
  (func $p10 (export "p10") (param i32 i32) (result i32)
    local.get 0
    i32.clz
    local.get 1
    i32.clz
    i32.const -2
    local.set 0
    i32.eq
  )
)
