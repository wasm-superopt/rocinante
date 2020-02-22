(module
  (func $use-local (export "use-local") (param i32) (result i32) (local i32)
    (local.set 1 
        (i32.mul 
            (local.get 0) (i32.const 2)))
    (i32.add 
        (local.get 0) (local.get 1))
  )
)

;; local.get 0
;; local.set 1
;; i32.const 3
;; i32.const -2
;; i32.rem_u
;; local.get 1
;; i32.mul

;; local.get 0
;; i32.const -2
;; local.get 0
;; i32.mul
;; i32.const 1
;; i32.div_u
;; i32.sub

;; local.get 0
;; local.get 0
;; local.get 0
;; i32.add
;; nop
;; local.tee 1
;; i32.add

;; local.get 0
;; i32.const 3
;; local.get 1
;; local.tee 1
;; i32.shl
;; nop
;; i32.mul

;; i32.const 3
;; local.get 0
;; nop
;; i32.mul
;; i32.const 1
;; local.tee 0
;; i32.div_u
