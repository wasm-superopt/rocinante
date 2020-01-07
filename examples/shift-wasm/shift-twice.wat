(module
  (type $t0 (func (param i32) (result i32)))
  (func $shift (type $t0) (param $p0 i32) (result i32)
    get_local $p0
    i32.const 1
    i32.shl
    i32.const 1
    i32.shl))