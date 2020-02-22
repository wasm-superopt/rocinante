;; Turn on the right-most 0-bit in a word.
(module
  (func $p6 (export "p6") (param i32) (result i32)
    (i32.or
      (local.get 0)
      (i32.add (local.get 0) (i32.const 1))
    )
  )
)