(module
  (type $t0 (func (param i32) (result i32)))
  (func $add (type $t0) (param $p0 i32) (result i32)
    local.get $p0
    local.get $p0
    i32.add)
  (export "add" (func $add))
)
