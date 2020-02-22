(module
  (type $t0 (func (param i32) (result i32)))
  (func $mul (type $t0) (param $p0 i32) (result i32)
    local.get $p0
    i32.const 3
    i32.mul)
  (export "mul" (func $mul)))
