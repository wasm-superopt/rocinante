(module 
    (func (export "shift-by-two") (param $x i32) (result i32)
        (i32.shl (local.get $x) (i32.const 2)))
    (func (export "shift-twice") (param $x i32) (result i32) 
        (i32.shl (i32.shl (local.get $x) (i32.const 1)) (i32.const 1)))
) 

(assert_return 
    (invoke "shift-by-two" (i32.const 1)) (i32.const 4))
(assert_return
    (invoke "shift-twice" (i32.const 1)) (i32.const 4))
(assert_return
    (invoke "shift-by-two" (i32.const 5))
    (i32.const 20))