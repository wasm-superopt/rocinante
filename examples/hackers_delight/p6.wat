;; Turn on the right-most 0-bit in a word.
(module
  (func $p6 (export "p6") (param i32) (result i32)
    (i32.or
      (get_local 0)
      (i32.add (get_local 0) (i32.const 1))
    )
  )
)