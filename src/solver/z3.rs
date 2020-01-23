use parity_wasm::elements::{FuncBody, FunctionType, Instruction, Local, ValueType};
use std::convert::TryFrom;
use std::fmt::Debug;
use z3::{ast, Context};

#[derive(Debug, Default)]
pub struct ValueStack<'ctx>(Vec<ast::Dynamic<'ctx>>);

impl<'ctx> ValueStack<'ctx> {
    pub fn new() -> Self {
        Self(Vec::new())
    }

    pub fn pop(&mut self) -> ast::Dynamic<'ctx> {
        self.0.pop().expect("stack empty")
    }

    pub fn pop_as<T: TryFrom<ast::Dynamic<'ctx>>>(&mut self) -> T
    where
        <T as TryFrom<z3::ast::Dynamic<'ctx>>>::Error: Debug,
    {
        T::try_from(self.pop()).unwrap()
    }

    pub fn pop_pair_as<T: TryFrom<ast::Dynamic<'ctx>>>(&mut self) -> (T, T)
    where
        <T as TryFrom<z3::ast::Dynamic<'ctx>>>::Error: Debug,
    {
        let lhs = self.pop_as::<T>();
        let rhs = self.pop_as::<T>();

        (lhs, rhs)
    }

    pub fn push<T: Into<ast::Dynamic<'ctx>>>(&mut self, val: T) {
        self.0.push(val.into());
    }
}

pub struct Converter<'ctx> {
    ctx: &'ctx Context,
    params: Vec<ast::Dynamic<'ctx>>,
    return_type: Option<ValueType>,
}

impl<'ctx> Converter<'ctx> {
    pub fn new(ctx: &'ctx Context, func_type: &FunctionType) -> Self {
        let mut params: Vec<ast::Dynamic<'ctx>> = Vec::with_capacity(func_type.params().len());

        for param in func_type.params() {
            match param {
                ValueType::I32 => params.push(ast::BV::fresh_const(&ctx, "p", 32).into()),
                ValueType::I64 => params.push(ast::BV::fresh_const(&ctx, "p", 64).into()),
                _ => {
                    panic!("float not supported.");
                }
            }
        }

        Self {
            ctx,
            params,
            return_type: func_type.return_type(),
        }
    }

    fn init_locals(&self, local_types: &[Local]) -> Vec<ast::Dynamic<'ctx>> {
        let mut locals = self.params.clone();
        locals.reserve(local_types.len());

        for local in local_types {
            let cnt = local.count();
            let local_type = local.value_type();

            for _ in 0..cnt {
                match local_type {
                    // Initial value of any local is 0.
                    ValueType::I32 => locals.push(ast::BV::from_u64(&self.ctx, 0, 32).into()),
                    ValueType::I64 => locals.push(ast::BV::from_u64(&self.ctx, 0, 64).into()),
                    _ => {
                        panic!("float not supported.");
                    }
                }
            }
        }

        locals
    }

    pub fn convert_func(&self, func: &FuncBody) -> Option<ast::Dynamic<'ctx>> {
        let mut locals = self.init_locals(func.locals());
        let mut stack: ValueStack<'ctx> = ValueStack::new();
        for instr in func.code().elements() {
            match instr {
                Instruction::I32Add => {
                    let (lhs, rhs) = stack.pop_pair_as::<ast::BV<'ctx>>();
                    let res = lhs.bvadd(&rhs);
                    stack.push(res);
                }
                Instruction::I32Sub => {
                    let (lhs, rhs) = stack.pop_pair_as::<ast::BV<'ctx>>();
                    let res = lhs.bvsub(&rhs);
                    stack.push(res);
                }
                Instruction::I32Mul => {
                    let (lhs, rhs) = stack.pop_pair_as::<ast::BV<'ctx>>();
                    let res = lhs.bvmul(&rhs);
                    stack.push(res);
                }
                Instruction::I32DivS => {
                    let (lhs, rhs) = stack.pop_pair_as::<ast::BV<'ctx>>();
                    let res = lhs.bvsdiv(&rhs);
                    stack.push(res);
                }
                Instruction::I32DivU => {
                    let (lhs, rhs) = stack.pop_pair_as::<ast::BV<'ctx>>();
                    let res = lhs.bvudiv(&rhs);
                    stack.push(res);
                }
                Instruction::I32RemS => {
                    let (lhs, rhs) = stack.pop_pair_as::<ast::BV<'ctx>>();
                    let res = lhs.bvsrem(&rhs);
                    stack.push(res);
                }
                Instruction::I32RemU => {
                    let (lhs, rhs) = stack.pop_pair_as::<ast::BV<'ctx>>();
                    let res = lhs.bvurem(&rhs);
                    stack.push(res);
                }
                Instruction::I32And => {
                    let (lhs, rhs) = stack.pop_pair_as::<ast::BV<'ctx>>();
                    let res = lhs.bvand(&rhs);
                    stack.push(res);
                }
                Instruction::I32Or => {
                    let (lhs, rhs) = stack.pop_pair_as::<ast::BV<'ctx>>();
                    let res = lhs.bvor(&rhs);
                    stack.push(res);
                }
                Instruction::I32Xor => {
                    let (lhs, rhs) = stack.pop_pair_as::<ast::BV<'ctx>>();
                    let res = lhs.bvxor(&rhs);
                    stack.push(res);
                }
                Instruction::I32Shl => {
                    let (lhs, rhs) = stack.pop_pair_as::<ast::BV<'ctx>>();
                    let res = lhs.bvshl(&rhs);
                    stack.push(res);
                }
                Instruction::I32ShrS => {
                    let (lhs, rhs) = stack.pop_pair_as::<ast::BV<'ctx>>();
                    // TODO: Confirm whether this is signed.
                    let res = lhs.bvashr(&rhs);
                    stack.push(res);
                }
                Instruction::I32ShrU => {
                    let (lhs, rhs) = stack.pop_pair_as::<ast::BV<'ctx>>();
                    // TODO: Confirm whether this is unsigned.
                    let res = lhs.bvlshr(&rhs);
                    stack.push(res);
                }
                Instruction::I32Rotl => {
                    let (lhs, rhs) = stack.pop_pair_as::<ast::BV<'ctx>>();
                    let res = lhs.bvrotl(&rhs);
                    stack.push(res);
                }
                Instruction::I32Rotr => {
                    let (lhs, rhs) = stack.pop_pair_as::<ast::BV<'ctx>>();
                    let res = lhs.bvrotr(&rhs);
                    stack.push(res);
                }
                Instruction::GetLocal(idx) => {
                    let val = &locals[*idx as usize];
                    stack.push(val.clone());
                }
                Instruction::SetLocal(idx) => {
                    let val = stack.pop();
                    locals[*idx as usize] = val;
                }
                Instruction::TeeLocal(idx) => {
                    let val = stack.pop();
                    stack.push(val.clone());
                    locals[*idx as usize] = val;
                }
                Instruction::I32Const(c) => {
                    let val = ast::BV::from_i64(&self.ctx, *c as i64, 32);
                    stack.push(val);
                }
                Instruction::End => {}
                _ => {
                    panic!("{} not supported", instr);
                }
            }
        }

        match self.return_type {
            Some(_) => Some(stack.pop()),
            None => None,
        }
    }
}
