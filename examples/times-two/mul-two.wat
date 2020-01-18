(module
  (type $t0 (func (param i32) (result i32)))
  (func $mul (type $t0) (param $p0 i32) (result i32)
    get_local $p0
    i32.const 2
    i32.mul)
  (export "mul" (func $mul)))
