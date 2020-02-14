;; Turn-off rightmost 1 bit.
(module
  (func $p1 (export "p1") (param i32) (result i32)
    i32.const -1
    local.get 0
    i32.add
    local.get 0
    i32.and
  )
)
