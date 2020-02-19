(module
  (func $use-local (export "use-local") (param i32) (result i32) (local i32)
    (local.set 1 
        (i32.mul 
            (local.get 0) (i32.const 2)))
    (i32.add 
        (local.get 0) (local.get 1))
  )
)

;; get_local 0
;; set_local 1
;; i32.const 3
;; i32.const -2
;; i32.rem_u
;; get_local 1
;; i32.mul

;; get_local 0
;; i32.const -2
;; get_local 0
;; i32.mul
;; i32.const 1
;; i32.div_u
;; i32.sub

;; get_local 0
;; get_local 0
;; get_local 0
;; i32.add
;; nop
;; tee_local 1
;; i32.add

;; get_local 0
;; i32.const 3
;; get_local 1
;; tee_local 1
;; i32.shl
;; nop
;; i32.mul

;; i32.const 3
;; get_local 0
;; nop
;; i32.mul
;; i32.const 1
;; tee_local 0
;; i32.div_u