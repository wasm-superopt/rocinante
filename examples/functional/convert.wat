(module
    (type $t0 (func (result i32)))
    ;; v0 = 1; 
    ;; return v0;
    (func $return-one (export "return-one") (type $t0) (result i32)
        i32.const 1)
    ;; v0 = p0;
    ;; return v0
    (type $t1 (func (param i32) (result i32)))
    (func $return-x (export "return-x") (type $t1) (param $p0 i32) (result i32)
        get_local $p0)
    ;; v0 = p0;
    ;; v1 = p0;
    ;; v2 = v0 + v1;
    ;; return v2;
    (func $binop (export "binop") (type $t1) (param $p0 i32)  (result i32)
        get_local $p0
        get_local $p0
        i32.add)
    ;; v0 = p0;
    ;; v1 = 1;
    ;; v2 = v0 << v1;
    ;; v3 = 1;
    ;; v4 = v2 << v3;
    ;; return v4;
    (func $shift-twice (export "shift-twice") (type $t1) (param $p0 i32) (result i32)
        get_local $p0
        i32.const 1
        i32.shl
        i32.const 1
        i32.shl)
    ;; Each local is intialized to 0.
    ;; l[0]0 = 0;
    ;; v0 = 0;
    ;; v1 = v0; // And also update that access to l[0] should just refer to v1.
    (func (export "type-local-i32") (local $l0 i32) (local.set 0 (i32.const 0)))

    (func (export "write") (param i32 i32) (result i32)
        ;; l[0, 1] initialized with arguments
        ;; l[2, 3] initialized with 0
        (local i32 i32)
        ;; v0 = 3;
        ;; v1 = v0; // update the local table to refer v1 for l[1]
        (local.set 1 (i32.const 3))
        ;; v2 = 40;
        ;; v3 = v2; // update the local table to refer  v3 for l[3]
        (local.set 3 (i32.const 40))

        ;; v4 = l[2];
        ;; v5 = v3;
        ;; v6 = v4 + v5;
        ;; v7 = v1 + v6;
        ;; v8 = l[0];
        ;; v9 = v8 + v7;
        (i32.add
        (local.get 0)
        (i32.add
            (local.get 1)
            (i32.add
            (local.get 2)
            (local.get 3)
            )
        )
        )
    )
)
