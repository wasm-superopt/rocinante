;; Turn-off the right-most contiguous string of 1 bits.
(module
  (func $p17 (export "p17") (param i32) (result i32)
    local.get 0
    local.get 0
    local.get 0
    i32.add
    local.get 0
    i32.or
    i32.sub
    local.get 0
    i32.and
  )
) 
