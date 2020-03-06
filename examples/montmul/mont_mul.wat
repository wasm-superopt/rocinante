(module 
 (func $mont_mul (export "mont_mul") (param $0 i64) (param $1 i64) (param $2 i32) (param $3 i32) (param $4 i64) (result i64)
  (local $5 i64)
  (local $6 i64)
  (local $7 i64)
  (i64.xor
   (i64.add
    (i64.add
     (i64.add
      (i64.add
       (select
        (i64.add
         (local.tee $5
          (i64.mul
           (local.tee $6
            (i64.shr_u
             (local.get $1)
             (i64.const 32)
            )
           )
           (local.tee $7
            (i64.extend_i32_u
             (local.get $3)
            )
           )
          )
         )
         (i64.const 4294967296)
        )
        (local.get $5)
        (i64.lt_u
         (local.tee $6
          (i64.add
           (local.tee $1
            (i64.mul
             (local.get $7)
             (local.tee $7
              (i64.and
               (local.get $1)
               (i64.const 4294967295)
              )
             )
            )
           )
           (i64.mul
            (local.get $6)
            (local.tee $5
             (i64.extend_i32_u
              (local.get $2)
             )
            )
           )
          )
         )
         (local.get $1)
        )
       )
       (i64.shr_u
        (local.get $6)
        (i64.const 32)
       )
      )
      (i64.extend_i32_u
       (i64.lt_u
        (local.tee $5
         (i64.add
          (local.tee $1
           (i64.shl
            (local.get $6)
            (i64.const 32)
           )
          )
          (i64.mul
           (local.get $5)
           (local.get $7)
          )
         )
        )
        (local.get $1)
       )
      )
     )
     (i64.extend_i32_u
      (i64.lt_u
       (local.tee $1
        (i64.add
         (local.get $4)
         (local.get $5)
        )
       )
       (local.get $5)
      )
     )
    )
    (i64.extend_i32_u
     (i64.lt_u
      (local.tee $0
       (i64.add
        (local.get $0)
        (local.get $1)
       )
      )
      (local.get $1)
     )
    )
   )
   (local.get $0)
  )
 )
)
