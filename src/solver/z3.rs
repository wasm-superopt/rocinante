use parity_wasm::elements::{FuncBody, FunctionType, Instruction, Local, ValueType};
use std::convert::TryFrom;
use std::fmt::Debug;
use z3::{ast, ast::Ast, Context, Solver};

// NOTE(taegyunkim): Consider also putting locals in value stack, and keeping a
// stack pointer to push and pop values.
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

#[derive(Debug)]
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

    pub fn bounds(&self) -> Vec<&ast::Dynamic<'ctx>> {
        self.params.iter().collect::<Vec<&ast::Dynamic<'ctx>>>()
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
                    // NOTE(taegyunkim): z3.rs doesn't provide wrappers for Z3
                    // floats. We first need to implement those in z3.rs
                    _ => {
                        panic!("float not supported.");
                    }
                }
            }
        }

        locals
    }

    // TODO(taegyunkim): Add test for each case.
    pub fn convert_func(&self, func: &FuncBody) -> ast::Dynamic<'ctx> {
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
                Instruction::End => {
                    // NOTE: no need to handle this for programs without loops
                    // and control structures.
                }
                _ => {
                    panic!("{} not supported", instr);
                }
            }
        }

        match self.return_type {
            Some(_) => stack.pop(),
            None => panic!("Doens't support void functions."),
        }
    }
}

#[derive(PartialEq, Eq, Debug)]
pub enum VerifyResult {
    Verified,
    CounterExample,
}

pub struct Z3Solver<'ctx> {
    ctx: &'ctx Context,
    converter: Converter<'ctx>,
    spec_f: ast::Dynamic<'ctx>,
}

impl<'ctx> Z3Solver<'ctx> {
    pub fn new(ctx: &'ctx Context, func_type: &FunctionType, spec: &FuncBody) -> Self {
        let converter = Converter::new(ctx, func_type);
        let spec_f = converter.convert_func(spec);
        Self {
            ctx,
            converter,
            spec_f,
        }
    }

    pub fn verify(&self, candidate: &FuncBody) -> VerifyResult {
        let candidate_f = self.converter.convert_func(candidate);
        let solver = Solver::new(&self.ctx);
        let bounds = self.converter.bounds();

        let forall = ast::forall_const(
            &self.ctx,
            &bounds,
            &[],
            &self.spec_f._eq(&candidate_f).into(),
        )
        .as_bool()
        .unwrap();

        solver.assert(&forall);

        match solver.check() {
            z3::SatResult::Sat => VerifyResult::Verified,
            z3::SatResult::Unsat => VerifyResult::CounterExample,
            z3::SatResult::Unknown => {
                panic!("Failed to prove or disprove.");
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{VerifyResult, Z3Solver};
    use crate::parity_wasm_utils;

    fn wat2module<S: AsRef<[u8]>>(source: S) -> parity_wasm::elements::Module {
        let binary = wabt::wat2wasm(source).expect("Failed to parse .wat");
        wasmparser::validate(&binary, None /* Uses default parser config */)
            .expect("Failed to validate.");
        parity_wasm::elements::Module::from_bytes(binary).expect("Failed to deserialize.")
    }

    #[test]
    fn verify_test() {
        let spec_module: parity_wasm::elements::Module = wat2module(
            r#"(module
                (type $t0 (func (param i32) (result i32)))
                (func $add (type $t0) (param $p0 i32) (result i32)
                  get_local $p0
                  get_local $p0
                  i32.add)
                (export "add" (func $add)))"#,
        );
        let (spec_func_type, spec_func_body) = parity_wasm_utils::func_by_name(&spec_module, "add");
        let candidate_module: parity_wasm::elements::Module = wat2module(
            r#"(module
                (type $t0 (func (param i32) (result i32)))
                (func $mul (type $t0) (param $p0 i32) (result i32)
                  get_local $p0
                  i32.const 2
                  i32.mul)
                (export "mul" (func $mul)))"#,
        );
        let (candidate_func_type, candidate_func_body) =
            parity_wasm_utils::func_by_name(&candidate_module, "mul");
        assert_eq!(spec_func_type, candidate_func_type);

        let cfg = z3::Config::new();
        let ctx = z3::Context::new(&cfg);

        let solver = Z3Solver::new(&ctx, spec_func_type, spec_func_body);
        assert_eq!(solver.verify(candidate_func_body), VerifyResult::Verified);
    }

    #[test]
    fn counterexample_test() {
        let spec_module: parity_wasm::elements::Module = wat2module(
            r#"(module
                (type $t0 (func (param i32) (result i32)))
                (func $add (type $t0) (param $p0 i32) (result i32)
                  get_local $p0
                  get_local $p0
                  i32.add)
                (export "add" (func $add)))"#,
        );
        let (spec_func_type, spec_func_body) = parity_wasm_utils::func_by_name(&spec_module, "add");
        let candidate_module: parity_wasm::elements::Module = wat2module(
            r#"(module
                (type $t0 (func (param i32) (result i32)))
                (func $mul (type $t0) (param $p0 i32) (result i32)
                  get_local $p0
                  i32.const 3
                  i32.mul)
                (export "mul" (func $mul)))"#,
        );
        let (candidate_func_type, candidate_func_body) =
            parity_wasm_utils::func_by_name(&candidate_module, "mul");
        assert_eq!(spec_func_type, candidate_func_type);

        let cfg = z3::Config::new();
        let ctx = z3::Context::new(&cfg);

        let solver = Z3Solver::new(&ctx, spec_func_type, spec_func_body);
        assert_eq!(
            solver.verify(candidate_func_body),
            VerifyResult::CounterExample
        );
    }
}
