;; Test whether an unsigned integer is of the form 2^(n-1)
(module
  (func $p2 (export "p2") (param i32) (result i32)
    local.get 0
    i32.const 1
    local.get 0
    i32.add
    i32.and
  )
)
