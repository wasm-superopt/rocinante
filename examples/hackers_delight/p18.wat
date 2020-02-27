;; P18(x) Determine if an integer is a power of 2 or not.
;; o_1 = bvsub(x, 1)
;; o_2 = bvand(o_1, x)
;; o_3 = bvredor (x)
;; o_4 = bvredor (o_2)
;; o_5 = !(o_4)
;; res = (o_5 && o_3)

;; bvredor -> reduction or equals to 0 iff all bits are 0
;; bvredor(x) == i32.ne (i32.const 0) (x)
;; if x == 0, then i32.ne (i32.const 0) => 0 otherwise 1.

;; !(bvredor(x)) == i32.eq (i32.const 0) (x)
;; if x = 0, then i32.eq (i32.const 0) (x) => 1 otherwise 0.

(module
  (func $p18 (export "p18") (param i32) (result i32)
    (i32.and 
      (i32.ne (i32.const 0) (local.get 0)) ;; o_3
      (i32.eq
        (i32.const 0)
        (i32.and 
          (i32.sub (local.get 0) (i32.const 1))
          (local.get 0)
        )
      )
    )
  )
)
