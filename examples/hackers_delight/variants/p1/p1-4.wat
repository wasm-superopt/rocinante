;; Turn-off rightmost 1 bit.
(module
  (func $p1 (export "p1") (param i32) (result i32)
    local.get 0
    local.get 0
    i32.const -1
    i32.add
    i32.and
  )
)
