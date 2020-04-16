(module
  (func $shift (export "shift-twice") (param $p0 i32) (result i32)
    local.get $p0
    i32.const 1
    i32.shl
    i32.const 1
    i32.shl))
