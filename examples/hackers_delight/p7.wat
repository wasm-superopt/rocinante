;; P7(x): Isolate the rightmost 0-bit.
;; o_1 = bvnot(x)
;; o_2 = bvadd(x, 1)
;; res := bvand(o_1, o_2)

;; [[(bvnot s)]] := λx:[0, m). if [[s]](x) = 0 then 1 else 0
;; for four bit numbers
;; bvnot 1111 = 0000
;; bvnot 0101 = 1010
;; bvnot 0000 = 1111

(module
  (func $p7 (export "p7") (param i32) (result i32)
    (i32.and
        (i32.xor (local.get 0) (i32.const -1))
        (i32.add (local.get 0) (i32.const 1))
    )
  )
)