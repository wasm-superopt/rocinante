;; Form a mask that identifies the rightmost 1 bit and trailing 0s.
(module
  (func $p4 (export "p4") (param i32) (result i32)
    (i32.xor
      (local.get 0)
      (i32.sub (local.get 0) (i32.const 1))
    )
  )
)