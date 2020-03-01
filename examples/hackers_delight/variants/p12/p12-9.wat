;; P12(x, y) Test if nlz(x) <= nlz(y)
;; where nlz is number of leading zeros.
;; o_1 = bvnot(x)
;; o_2 = bvand(y, o_1)
;; res := bvule(o_2, x)

(module
  (func $p12 (export "p12") (param i32 i32) (result i32)
    local.get 0
    nop
    i32.clz
    local.get 1
    nop
    i32.clz
    i32.le_u
  )
)
