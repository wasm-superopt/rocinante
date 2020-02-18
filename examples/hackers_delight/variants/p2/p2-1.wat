;; Test whether an unsigned integer is of the form 2^(n-1)
(module
  (func $p2 (export "p2") (param i32) (result i32)
    local.get 0
    local.get 0
    i32.const -1
    i32.sub
    i32.and
  )
)
