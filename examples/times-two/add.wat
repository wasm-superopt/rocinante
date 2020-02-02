(module
  (type $t0 (func (param i32) (result i32)))
  (func $add (type $t0) (param $p0 i32) (result i32)
    get_local $p0
    get_local $p0
    i32.add)
  (export "add" (func $add))
)
