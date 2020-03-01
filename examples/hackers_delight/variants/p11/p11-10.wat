;; P11(x, y) Test if nlz(x) < nlz(y)
;; where nlz is number of leading zeros.
;; o_1 = bvnot(y)
;; o_2 = bvand(x, o_1)
;; res := bvugt(o_2, y)

(module
  (func $p11 (export "p11") (param i32 i32) (result i32)
    i32.const -1
    local.get 0
    i32.clz
    local.get 1
    i32.clz
    i32.lt_u
    i32.and
  )
)
