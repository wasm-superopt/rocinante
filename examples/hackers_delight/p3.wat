;; Isolate the rightmost 1 bit.
(module
  (func $p3 (export "p3") (param i32) (result i32)
    (i32.and
      (i32.sub (i32.const 0) (get_local 0))
      (get_local 0))
  )
)