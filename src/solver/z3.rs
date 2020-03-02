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
        let rhs = self.pop_as::<T>();
        let lhs = self.pop_as::<T>();

        (lhs, rhs)
    }

    pub fn push<T: Into<ast::Dynamic<'ctx>>>(&mut self, val: T) {
        self.0.push(val.into());
    }
}

#[derive(Debug)]
pub struct Converter<'ctx> {
    ctx: &'ctx Context,
    z3_params: Vec<ast::Dynamic<'ctx>>,
    func_type: FunctionType,
}

fn ctz<'a>(ctx: &'a Context, input: &ast::BV<'a>) -> ast::BV<'a> {
    let one_bit = ast::BV::from_u64(ctx, 1, 1);

    fn ctz_helper<'a>(
        ctx: &'a z3::Context,
        input: &ast::BV<'a>,
        one_bit: &ast::BV<'a>,
        i: u32,
    ) -> ast::BV<'a> {
        let bit_width = input.get_size();

        if i == bit_width {
            ast::BV::from_u64(ctx, i as u64, bit_width)
        } else {
            input.extract(i, i)._eq(&one_bit).ite(
                &ast::BV::from_u64(ctx, i as u64, bit_width),
                &ctz_helper(ctx, input, one_bit, i + 1),
            )
        }
    }

    ctz_helper(ctx, input, &one_bit, 0)
}

fn clz<'a>(ctx: &'a Context, input: &ast::BV<'a>) -> ast::BV<'a> {
    let one_bit = ast::BV::from_u64(ctx, 1, 1);

    fn clz_helper<'a>(
        ctx: &'a z3::Context,
        input: &ast::BV<'a>,
        one_bit: &ast::BV<'a>,
        i: u32,
    ) -> ast::BV<'a> {
        let bit_width = input.get_size();
        if i == bit_width {
            ast::BV::from_u64(ctx, i as u64, bit_width)
        } else {
            input
                .extract(bit_width - 1 - i, bit_width - 1 - i)
                ._eq(&one_bit)
                .ite(
                    &ast::BV::from_u64(ctx, i as u64, bit_width),
                    &clz_helper(ctx, input, one_bit, i + 1),
                )
        }
    }

    clz_helper(ctx, input, &one_bit, 0)
}

fn popcnt<'a>(ctx: &'a Context, input: &ast::BV<'a>) -> ast::BV<'a> {
    // As in https://stackoverflow.com/questions/39299015/sum-of-all-the-bits-in-a-bit-vector-of-z3
    let bit_width = input.get_size();
    let bits: Vec<ast::BV<'a>> = (0..bit_width).map(|i| input.extract(i, i)).collect();
    let bvs: Vec<ast::BV<'a>> = bits
        .into_iter()
        .map(|b| ast::BV::from_u64(ctx, 0, bit_width - 1).concat(&b))
        .collect();
    bvs.into_iter()
        .fold(ast::BV::from_u64(ctx, 0, bit_width), |acc, x| acc.bvadd(&x))
}

impl<'ctx> Converter<'ctx> {
    pub fn new(ctx: &'ctx Context, func_type: &FunctionType) -> Self {
        let mut z3_params: Vec<ast::Dynamic<'ctx>> = Vec::with_capacity(func_type.params().len());

        for param in func_type.params() {
            match param {
                ValueType::I32 => z3_params.push(ast::BV::fresh_const(&ctx, "p", 32).into()),
                ValueType::I64 => z3_params.push(ast::BV::fresh_const(&ctx, "p", 64).into()),
                _ => {
                    panic!("float not supported.");
                }
            }
        }

        Self {
            ctx,
            z3_params,
            func_type: func_type.clone(),
        }
    }

    pub fn bounds(&self) -> Vec<&ast::Dynamic<'ctx>> {
        self.z3_params.iter().collect::<Vec<&ast::Dynamic<'ctx>>>()
    }

    fn init_locals(&self, local_types: &[Local]) -> Vec<ast::Dynamic<'ctx>> {
        let mut locals = self.z3_params.clone();
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
                // I32 binops
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
                    // NOTE(taegyunkim): sign-replicating (arithmetic) shift right.
                    let (lhs, rhs) = stack.pop_pair_as::<ast::BV<'ctx>>();
                    let res = lhs.bvashr(&rhs);
                    stack.push(res);
                }
                Instruction::I32ShrU => {
                    // NOTE(taegyunkim): zero-replicating (logical) shift right.
                    let (lhs, rhs) = stack.pop_pair_as::<ast::BV<'ctx>>();
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
                // local variable ops
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
                    // stack.push(val.clone());
                    // val = stack.pop();
                    locals[*idx as usize] = val;
                }
                Instruction::I32Const(c) => {
                    let val = ast::BV::from_i64(&self.ctx, *c as i64, 32);
                    stack.push(val);
                }
                // I32 relops
                Instruction::I32Eq => {
                    let (lhs, rhs) = stack.pop_pair_as::<ast::BV<'ctx>>();
                    let res = lhs._eq(&rhs);
                    stack.push(res.ite(
                        &ast::BV::from_i64(&self.ctx, 1, 32),
                        &ast::BV::from_i64(&self.ctx, 0, 32),
                    ));
                }
                Instruction::I32Ne => {
                    let (lhs, rhs) = stack.pop_pair_as::<ast::BV<'ctx>>();
                    let res = lhs._eq(&rhs);
                    stack.push(res.ite(
                        &ast::BV::from_i64(&self.ctx, 0, 32),
                        &ast::BV::from_i64(&self.ctx, 1, 32),
                    ));
                }
                Instruction::I32LtS => {
                    let (lhs, rhs) = stack.pop_pair_as::<ast::BV<'ctx>>();
                    let res = lhs.bvslt(&rhs);
                    stack.push(res.ite(
                        &ast::BV::from_i64(&self.ctx, 1, 32),
                        &ast::BV::from_i64(&self.ctx, 0, 32),
                    ));
                }
                Instruction::I32LtU => {
                    let (lhs, rhs) = stack.pop_pair_as::<ast::BV<'ctx>>();
                    let res = lhs.bvult(&rhs);
                    stack.push(res.ite(
                        &ast::BV::from_i64(&self.ctx, 1, 32),
                        &ast::BV::from_i64(&self.ctx, 0, 32),
                    ));
                }
                Instruction::I32GtS => {
                    let (lhs, rhs) = stack.pop_pair_as::<ast::BV<'ctx>>();
                    let res = lhs.bvsgt(&rhs);
                    stack.push(res.ite(
                        &ast::BV::from_i64(&self.ctx, 1, 32),
                        &ast::BV::from_i64(&self.ctx, 0, 32),
                    ));
                }
                Instruction::I32GtU => {
                    let (lhs, rhs) = stack.pop_pair_as::<ast::BV<'ctx>>();
                    let res = lhs.bvugt(&rhs);
                    stack.push(res.ite(
                        &ast::BV::from_i64(&self.ctx, 1, 32),
                        &ast::BV::from_i64(&self.ctx, 0, 32),
                    ));
                }
                Instruction::I32LeS => {
                    let (lhs, rhs) = stack.pop_pair_as::<ast::BV<'ctx>>();
                    let res = lhs.bvsle(&rhs);
                    stack.push(res.ite(
                        &ast::BV::from_i64(&self.ctx, 1, 32),
                        &ast::BV::from_i64(&self.ctx, 0, 32),
                    ));
                }
                Instruction::I32LeU => {
                    let (lhs, rhs) = stack.pop_pair_as::<ast::BV<'ctx>>();
                    let res: ast::Bool<'ctx> = lhs.bvule(&rhs);
                    stack.push(res.ite(
                        &ast::BV::from_i64(&self.ctx, 1, 32),
                        &ast::BV::from_i64(&self.ctx, 0, 32),
                    ));
                }
                Instruction::I32GeS => {
                    let (lhs, rhs) = stack.pop_pair_as::<ast::BV<'ctx>>();
                    let res = lhs.bvsge(&rhs);
                    stack.push(res.ite(
                        &ast::BV::from_i64(&self.ctx, 1, 32),
                        &ast::BV::from_i64(&self.ctx, 0, 32),
                    ));
                }
                Instruction::I32GeU => {
                    let (lhs, rhs) = stack.pop_pair_as::<ast::BV<'ctx>>();
                    let res = lhs.bvuge(&rhs);
                    stack.push(res.ite(
                        &ast::BV::from_i64(&self.ctx, 1, 32),
                        &ast::BV::from_i64(&self.ctx, 0, 32),
                    ));
                }
                // i32 testop
                Instruction::I32Eqz => {
                    let val = stack.pop_as::<ast::BV<'ctx>>();
                    let res = val._eq(&ast::BV::from_i64(&self.ctx, 0, 32));
                    stack.push(res.ite(
                        &ast::BV::from_i64(&self.ctx, 1, 32),
                        &ast::BV::from_i64(&self.ctx, 0, 32),
                    ));
                }
                // i32 unops
                Instruction::I32Clz => {
                    let val = stack.pop_as::<ast::BV<'ctx>>();
                    stack.push(clz(&self.ctx, &val));
                }
                Instruction::I32Ctz => {
                    let val = stack.pop_as::<ast::BV<'ctx>>();
                    stack.push(ctz(&self.ctx, &val));
                }
                Instruction::I32Popcnt => {
                    let val = stack.pop_as::<ast::BV<'ctx>>();
                    stack.push(popcnt(&self.ctx, &val));
                }
                // control instructions
                Instruction::Nop => {
                    // Do nothing
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

        match self.func_type.return_type() {
            Some(_) => stack.pop(),
            None => panic!("Doens't support void functions."),
        }
    }
}

#[derive(PartialEq, Debug)]
pub enum VerifyResult {
    Verified,
    // Use wasmi::RuntimeValue to make it easier to handle these instead of
    // defining a new one.
    CounterExample(Vec<wasmer_runtime::Value>),
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

        solver.assert(&self.spec_f._eq(&candidate_f).not());

        match solver.check() {
            z3::SatResult::Sat => {
                let model = solver.get_model();

                let mut values = Vec::new();

                for (i, bound) in self.converter.bounds().iter().enumerate() {
                    let typ = self.converter.func_type.params()[i];

                    match typ {
                        ValueType::I32 => {
                            values.push(wasmer_runtime::Value::I32(
                                model
                                    .eval(&bound.as_bv().unwrap())
                                    .unwrap()
                                    .as_i64()
                                    .unwrap() as i32,
                            ));
                        }
                        ValueType::I64 => {
                            values.push(wasmer_runtime::Value::I64(
                                model
                                    .eval(&bound.as_bv().unwrap())
                                    .unwrap()
                                    .as_i64()
                                    .unwrap(),
                            ));
                        }
                        unexpected => panic!("{} not supported", unexpected),
                    }
                }

                VerifyResult::CounterExample(values)
            }
            z3::SatResult::Unsat => VerifyResult::Verified,
            z3::SatResult::Unknown => {
                panic!("Failed to prove or disprove.");
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parity_wasm_utils;

    fn wat2module<S: AsRef<[u8]>>(source: S) -> parity_wasm::elements::Module {
        let binary = wabt::wat2wasm(source).expect("Failed to parse .wat");
        wasmparser::validate(&binary, None /* Uses default parser config */)
            .expect("Failed to validate.");
        parity_wasm::elements::Module::from_bytes(binary).expect("Failed to deserialize.")
    }

    fn instantiate(module: parity_wasm::elements::Module) -> wasmi::ModuleRef {
        let module =
            wasmi::Module::from_parity_wasm_module(module).expect("Failed to load wasmi module.");
        wasmi::ModuleInstance::new(&module, &wasmi::ImportsBuilder::default())
            .expect("Failed to build wasmi module instance.")
            .assert_no_start()
    }

    fn to_wasmi_values(values: Vec<wasmer_runtime::Value>) -> Vec<wasmi::RuntimeValue> {
        values
            .into_iter()
            .map(|v| match v {
                ::wasmer_runtime::Value::I32(x) => wasmi::RuntimeValue::I32(x),
                unimplemented => panic!("type not implemented {:?}", unimplemented),
            })
            .collect()
    }

    // Verifies that x + x == 2 * x.
    #[test]
    fn verify_test() {
        let spec_module: parity_wasm::elements::Module = wat2module(
            r#"(module
                (type $t0 (func (param i32) (result i32)))
                (func $add (type $t0) (param $p0 i32) (result i32)
                  local.get $p0
                  local.get $p0
                  i32.add)
                (export "add" (func $add)))"#,
        );
        let (spec_func_type, spec_func_body) = parity_wasm_utils::func_by_name(&spec_module, "add");
        let candidate_module: parity_wasm::elements::Module = wat2module(
            r#"(module
                (type $t0 (func (param i32) (result i32)))
                (func $mul (type $t0) (param $p0 i32) (result i32)
                  local.get $p0
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

    // Verifies that x + x == x << 1.
    #[test]
    fn verify_shl_test() {
        let spec_module: parity_wasm::elements::Module = wat2module(
            r#"(module
                (type $t0 (func (param i32) (result i32)))
                (func $add (type $t0) (param $p0 i32) (result i32)
                  local.get $p0
                  local.get $p0
                  i32.add)
                (export "add" (func $add)))"#,
        );
        let (spec_func_type, spec_func_body) = parity_wasm_utils::func_by_name(&spec_module, "add");
        let candidate_module: parity_wasm::elements::Module = wat2module(
            r#"(module
                (type $t0 (func (param i32) (result i32)))
                (func $shl (type $t0) (param $p0 i32) (result i32)
                  local.get $p0
                  i32.const 1
                  i32.shl)
                (export "shl" (func $shl)))"#,
        );
        let (candidate_func_type, candidate_func_body) =
            parity_wasm_utils::func_by_name(&candidate_module, "shl");
        assert_eq!(spec_func_type, candidate_func_type);

        let cfg = z3::Config::new();
        let ctx = z3::Context::new(&cfg);

        let solver = Z3Solver::new(&ctx, spec_func_type, spec_func_body);
        assert_eq!(solver.verify(candidate_func_body), VerifyResult::Verified);
    }

    // Checks that x + x != x * 3 and generates a counterexample. Then, using
    // wasmi, WASM interpreter, the generated counterexample indeed results in
    // different values when applied to two functions.
    #[test]
    fn counterexample_test() {
        let spec_module: parity_wasm::elements::Module = wat2module(
            r#"(module
                (type $t0 (func (param i32) (result i32)))
                (func $add (type $t0) (param $p0 i32) (result i32)
                  local.get $p0
                  local.get $p0
                  i32.add)
                (export "add" (func $add)))"#,
        );
        let (spec_func_type, spec_func_body) = parity_wasm_utils::func_by_name(&spec_module, "add");
        let candidate_module: parity_wasm::elements::Module = wat2module(
            r#"(module
                (type $t0 (func (param i32) (result i32)))
                (func $mul (type $t0) (param $p0 i32) (result i32)
                  local.get $p0
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
        let result = solver.verify(candidate_func_body);
        assert_matches!(result, VerifyResult::CounterExample(_));
        if let VerifyResult::CounterExample(cex_vec) = result {
            assert_eq!(cex_vec.len(), 1);

            let cex_vec = to_wasmi_values(cex_vec);

            let spec_instance = instantiate(spec_module);
            let spec_output = spec_instance
                .invoke_export("add", &cex_vec, &mut wasmi::NopExternals)
                .unwrap();

            let candidate_instance = instantiate(candidate_module);
            let candidate_output = candidate_instance
                .invoke_export("mul", &cex_vec, &mut wasmi::NopExternals)
                .unwrap();
            assert_ne!(spec_output, candidate_output);

            // Pass --nocapture to check following output.
            // $> cargo test -- --nocapture
            // [I32(-1)] Some(I32(-2)) Some(I32(-3))
            println!("{:?} {:?} {:?}", cex_vec, spec_output, candidate_output);
        }
    }
    #[test]
    fn ctz_test() {
        let _ = env_logger::try_init();
        let cfg = z3::Config::new();
        let ctx = z3::Context::new(&cfg);

        // 0000 0000 0000 0000 0000 0000 0000 0010
        assert!(ctz(&ctx, &ast::BV::from_i64(&ctx, 2, 32))
            ._eq(&ast::BV::from_u64(&ctx, 1, 32))
            .simplify()
            .as_bool()
            .unwrap());
        // all zeroes
        assert!(ctz(&ctx, &ast::BV::from_i64(&ctx, 0, 32))
            ._eq(&ast::BV::from_i64(&ctx, 32, 32))
            .simplify()
            .as_bool()
            .unwrap());
        // all ones
        assert!(ctz(&ctx, &ast::BV::from_i64(&ctx, -1, 32))
            ._eq(&ast::BV::from_i64(&ctx, 0, 32))
            .simplify()
            .as_bool()
            .unwrap());
        // 00 1010
        assert!(ctz(&ctx, &ast::BV::from_i64(&ctx, 10, 6))
            ._eq(&ast::BV::from_i64(&ctx, 1, 6))
            .simplify()
            .as_bool()
            .unwrap());
    }

    #[test]
    fn clz_test() {
        let _ = env_logger::try_init();
        let cfg = z3::Config::new();
        let ctx = z3::Context::new(&cfg);

        // 0000 0000 0000 0000 0000 0000 0000 0010
        assert!(clz(&ctx, &ast::BV::from_i64(&ctx, 2, 32))
            ._eq(&ast::BV::from_u64(&ctx, 30, 32))
            .simplify()
            .as_bool()
            .unwrap());
        // all zeroes
        assert!(clz(&ctx, &ast::BV::from_i64(&ctx, 0, 32))
            ._eq(&ast::BV::from_i64(&ctx, 32, 32))
            .simplify()
            .as_bool()
            .unwrap());
        // all ones
        assert!(clz(&ctx, &ast::BV::from_i64(&ctx, -1, 32))
            ._eq(&ast::BV::from_i64(&ctx, 0, 32))
            .simplify()
            .as_bool()
            .unwrap());
        // 00 1010
        assert!(clz(&ctx, &ast::BV::from_i64(&ctx, 10, 6))
            ._eq(&ast::BV::from_i64(&ctx, 2, 6))
            .simplify()
            .as_bool()
            .unwrap());
    }

    #[test]
    fn popcnt_test() {
        let _ = env_logger::try_init();
        let cfg = z3::Config::new();
        let ctx = z3::Context::new(&cfg);

        // 0000 0000 0000 0000 0000 0000 0000 0010
        assert!(popcnt(&ctx, &ast::BV::from_i64(&ctx, 2, 32))
            ._eq(&ast::BV::from_u64(&ctx, 1, 32))
            .simplify()
            .as_bool()
            .unwrap());
        // all zerores
        assert!(popcnt(&ctx, &ast::BV::from_i64(&ctx, 0, 32))
            ._eq(&ast::BV::from_i64(&ctx, 0, 32))
            .simplify()
            .as_bool()
            .unwrap());
        // all ones
        assert!(popcnt(&ctx, &ast::BV::from_i64(&ctx, -1, 32))
            ._eq(&ast::BV::from_i64(&ctx, 32, 32))
            .simplify()
            .as_bool()
            .unwrap());
        // 00 1010
        assert!(popcnt(&ctx, &ast::BV::from_i64(&ctx, 10, 6))
            ._eq(&ast::BV::from_i64(&ctx, 2, 6))
            .simplify()
            .as_bool()
            .unwrap());
    }
}
