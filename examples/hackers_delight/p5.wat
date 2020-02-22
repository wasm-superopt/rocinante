;; Right propagate rightmost 1 bit.
(module
  (func $p5 (export "p5") (param i32) (result i32)
    (i32.or
      (local.get 0)
      (i32.sub (local.get 0) (i32.const 1))
    )
  )
)