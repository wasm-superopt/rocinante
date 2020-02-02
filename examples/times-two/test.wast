(module
    (type $t0 (func (param i32) (result i32)))
    (func $shl (type $t0) (param $p0 i32) (result i32)
        (i32.shl (get_local $p0) (i32.const 1))
    )
    (export "shl" (func $shl))
    (func $add (type $t0) (param $p0 i32) (result i32)
        (i32.add (get_local $p0) (get_local $p0))
    )
    (export "add" (func $add))
)

(assert_return 
    (invoke "add" (i32.const 453194490)) (invoke "shl" (i32.const 453194490)))