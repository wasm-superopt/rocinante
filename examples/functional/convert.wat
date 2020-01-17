(module
    (type $t0 (func (result i32)))
    ;; v0 = 1; 
    ;; return v0;
    ;; 
    ;; will translate back to
    ;; i32.const 1
    ;; local.set 0
    ;; local.get 0
    (func $return-one (export "return-one") (type $t0) (result i32)
        i32.const 1)
    ;; v1 = v0;
    ;; return v1
    ;; 
    ;; will translate back to
    ;; local.get 0
    ;; local.set 1
    ;; local.get 1
    (type $t1 (func (param i32) (result i32)))
    (func $return-x (export "return-x") (type $t1) (param $p0 i32) (result i32)
        local.get $p0)
    ;; v1 = v0;
    ;; v2 = v0;
    ;; v3 = v1 + v2;
    ;; return v3;
    ;; 
    ;; will translate back to
    ;; local.get 0
    ;; local.set 1
    ;; local.get 0
    ;; local.set 2
    ;; local.get 1
    ;; local.get 2
    ;; i32.add
    ;; local.set 3
    ;; local.get 3
    (func $binop (export "binop") (type $t1) (param $p0 i32)  (result i32)
        local.get $p0
        local.get $p0
        i32.add)
    ;; v1 = v0;
    ;; v2 = v0;
    ;; v3 = v1 + v2;
    ;; v4 = v3; // Now local[0] will point to v4
    ;; v5 = v4;
    ;; v6 = v4;
    ;; v7 = v5 + v6;
    ;; return v7;
    ;;
    ;; will translate back to
    ;; local.get 0
    ;; local.set 1
    ;; local.get 0
    ;; local.set 2
    ;; local.get 1
    ;; local.get 2
    ;; i32.add
    ;; local.set 3
    ;; local.get 3
    ;; local.set 4
    ;; local.get 4
    ;; local.set 5
    ;; local.get 4
    ;; local.set 6
    ;; local.get 5
    ;; local.get 6
    ;; i32.add
    ;; local.set 7
    ;; local.get 7
    (func $get-set (export "get-set") (type $t1) (param $p0 i32)  (result i32)
        local.get $p0
        local.get $p0
        i32.add
        local.set $p0
        local.get $p0
        local.get $p0
        i32.add
        )
    (func $get-set-trans (export "get-set-trans") (type $t1) (param $p0 i32) (result i32)
    (local i32 i32 i32 i32 i32 i32 i32) 
        local.get 0
        local.set 1
        local.get 0
        local.set 2
        local.get 1
        local.get 2
        i32.add
        local.set 3
        local.get 3
        local.set 4
        local.get 4
        local.set 5
        local.get 4
        local.set 6
        local.get 5
        local.get 6
        i32.add
        local.set 7
        local.get 7)
    ;; v0 = p0;
    ;; v1 = 1;
    ;; v2 = v0 << v1;
    ;; v3 = 1;
    ;; v4 = v2 << v3;
    ;; return v4;
    (func $shift-twice (export "shift-twice") (type $t1) (param $p0 i32) (result i32)
        local.get $p0
        i32.const 1
        i32.shl
        i32.const 1
        i32.shl)
    ;; Each local is intialized to 0.
    ;; v0 = 0;
    ;; v1 = v0; // And also update that access to l[0] should just refer to v1.
    (func (export "type-local-i32") (local $l0 i32) (local.set 0 (i32.const 0)))

    (func (export "write") (param i32 i32) (result i32)
        ;; l[0, 1] initialized with arguments
        ;; l[2, 3] initialized with 0
        (local i32 i32)
        ;; v0 = 3;
        ;; v1 = v0; // update the local table to refer v1 for l[1]
        i32.const 3
        local.set 1
        ;; v2 = 40;
        ;; v3 = v2; // update the local table to refer  v3 for l[3]
        i32.const 40
        local.set 3

        ;; v4 = l[0];
        local.get 0
        ;; v5 = v1;
        local.get 1
        ;; v6 = l[2];
        local.get 2
        ;; v7 = v3;
        local.get 3
        ;; v8 = v6 + v7;
        i32.add
        ;; v9 = v5 + v8;
        i32.add
        ;; v10 = v4 + v9;
        i32.add
        ;; return v10;

        ;; (i32.add
        ;; (local.get 0)
        ;; (i32.add
        ;;     (local.get 1)
        ;;     (i32.add
        ;;     (local.get 2)
        ;;     (local.get 3)
        ;;     )
        ;; )
        ;; )
    )

    (func (export "empty") (param i32)
        (if (local.get 0) (then))
        (if (local.get 0) (then) (else))
        (if $l (local.get 0) (then))
        (if $l (local.get 0) (then) (else))
        ;; local.get 0
        ;; if 
        ;; end
        ;; local.get 0
        ;; if
        ;; end
        ;; local.get 0
        ;; if
        ;; end
        ;; local.get 0
        ;; if
        ;; end

        ;; v0 = l[0];
        ;; if v0 == 0 {
        ;; }
        ;; v1 = l[0];
        ;; if v1 == 0 {
        ;; }
        ;; v2 = l[0];
        ;; if v1 == 0 {
        ;; }
        ;; v3 = l[0];
        ;; if v1 == 0 {
        ;; }
    )

)
