(module
    (type $t0 (func (param i32) (result i32)))
        (func $shl (type $t0) (param $p0 i32) (result i32)
            local.get $p0
            i32.const 1
            i32.shl)
    (export "shl" (func $shl)))