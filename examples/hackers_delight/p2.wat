;; Test whether an unsigned integer is of the form 2^(n-1)
(module
  (func $p2 (export "p2") (param i32) (result i32)
    (i32.and
      (i32.add (get_local 0) (i32.const 1))
      (get_local 0))
  )
)