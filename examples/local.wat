(module
  (func $main (export "main") (result i32) (local i32)
    (local.set 0 (i32.const 10))
    (i32.add (call $localpalooza) (local.get 0))
  )
  (func $localpalooza (export "localpalooza") (result i32) (local i32 i32 i32 i32 i32 i32 i32 i32)

    (local.set 0 (i32.const 1))
    (local.set 1 (i32.const 1))
    (local.set 2 (i32.const 1))
    (local.set 3 (i32.const 1))
    (local.set 4 (i32.const 1))
    (local.set 5 (i32.const 1))
    (local.set 6 (i32.const 1))
    (local.set 7 (i32.const 1))

    (local.set 0
      (i32.add (call $localpalooza2) (local.get 0)))

    (local.set 1 (i32.add (local.get 0) (local.get 1)))
    (local.set 2 (i32.add (local.get 1) (local.get 2)))
    (local.set 3 (i32.add (local.get 2) (local.get 3)))
    (local.set 4 (i32.add (local.get 3) (local.get 4)))
    (local.set 5 (i32.add (local.get 4) (local.get 5)))
    (local.set 6 (i32.add (local.get 5) (local.get 6)))
    (local.set 7 (i32.add (local.get 6) (local.get 7)))

    (local.get 7)
  )

  (func $localpalooza2 (export "localpalooza2") (result i32) (local i32 i32 i32 i32 i32 i32 i32 i32)

    (local.set 0 (i32.const 2))
    (local.set 1 (i32.const 2))
    (local.set 2 (i32.const 2))
    (local.set 3 (i32.const 2))
    (local.set 4 (i32.const 2))
    (local.set 5 (i32.const 2))
    (local.set 6 (i32.const 2))
    (local.set 7 (i32.const 2))

    (local.set 1 (i32.add (local.get 0) (local.get 1)))
    (local.set 2 (i32.add (local.get 1) (local.get 2)))
    (local.set 3 (i32.add (local.get 2) (local.get 3)))
    (local.set 4 (i32.add (local.get 3) (local.get 4)))
    (local.set 5 (i32.add (local.get 4) (local.get 5)))
    (local.set 6 (i32.add (local.get 5) (local.get 6)))
    (local.set 7 (i32.add (local.get 6) (local.get 7)))

    (local.get 7)
  )

  (func $different_types (export "different_types") (result i32) (local i32 i64 i32)
    (local.set 0 (i32.const 2))
    (local.set 1 (i64.const 1))
    (local.set 2 (i32.const 3))

    (local.get 2)
  )

)
