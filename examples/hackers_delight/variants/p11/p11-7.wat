;; P11(x, y) Test if nlz(x) < nlz(y)
;; where nlz is number of leading zeros.
;; o_1 = bvnot(y)
;; o_2 = bvand(x, o_1)
;; res := bvugt(o_2, y)

(module
  (func $p11 (export "p11") (param i32 i32) (result i32)
    local.get 0
    i32.clz
    local.get 1
    local.get 1
    local.set 0
    i32.clz
    i32.lt_s
  )
)
