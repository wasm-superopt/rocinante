;; P8(x): Form a mask that identifies the trailing 0’s
;; o_1 = bvsub(x, 1)
;; o_2 = bvnot(x)
;; res := bvand(o_1, o_2)

(module
  (func $p8 (export "p8") (param i32) (result i32)
    (i32.and
        (i32.sub (local.get 0) (i32.const 1))
        (i32.xor (local.get 0) (i32.const -1))
    )
  )
)