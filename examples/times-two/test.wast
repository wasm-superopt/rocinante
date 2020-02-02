(module
    (type $t0 (func (param i32) (result i32)))
    (func $shl (type $t0) (param i32) (result i32)
        (i32.shl (local.get 0) (i32.const 1))
    )
    (export "shl" (func $shl))
    (func $add (type $t0) (param i32) (result i32)
        (i32.add (local.get 0) (local.get 0))
    )
    (export "add" (func $add))
)

(assert_return 
    (invoke "add" (i32.const 453194490)) (i32.const 906388980))
(assert_return 
    (invoke "shl" (i32.const 453194490)) (i32.const 906388980))
