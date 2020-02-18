;; Turn-off the right-most contiguous string of 1 bits.
(module
  (func $p17 (export "p17") (param i32) (result i32)
    local.get 0
    local.get 0
    local.get 0
    i32.const -1
    i32.div_s
    local.get 0
    i32.and
    i32.add
    i32.and
  )
) 
