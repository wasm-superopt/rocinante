(module 
 # Define a function that takes 2 i32 and return a i32
 (type $0 (func (param i32 i32) (result i32)))
 (memory $0 0)
 # Export the first function defined in the type section with name "add"
 (export "add" (func $0))
 # The body of the function defined above
 (func $0 (type $0) (param $var$0 i32) (param $var$1 i32) (result i32) 
  # The opcode that stands for addition of i32
  (i32.add
   (local.get $var$0) # load first param
   (local.get $var$1) # load second param
  )
 )
) 
