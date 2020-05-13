use parity_wasm::elements::{FuncBody, FunctionType, Instruction, Local, ValueType};
use std::convert::TryFrom;
use std::fmt::Debug;
use std::collections::HashMap;
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
    /// Z3 Context
    ctx: &'ctx Context,
    /// Z3 variable representing the parameters of spec function.
    z3_params: Vec<ast::Dynamic<'ctx>>,
    /// Spec function type
    func_type: FunctionType,
    /// Local variable types, doesn't include the parameters though they're also accessed via the
    /// same `local.{get, set, tee}` instructions.
    local_types: Vec<ValueType>,
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
    pub fn new(ctx: &'ctx Context, func_type: &FunctionType, locals: &[Local]) -> Self {
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
        let mut local_types = Vec::new();

        for local in locals {
            let cnt = local.count();
            let local_type = local.value_type();

            for _ in 0..cnt {
                local_types.push(local_type);
            }
        }
        Self {
            ctx,
            z3_params,
            func_type: func_type.clone(),
            local_types,
        }
    }

    pub fn bounds(&self) -> Vec<&ast::Dynamic<'ctx>> {
        self.z3_params.iter().collect::<Vec<&ast::Dynamic<'ctx>>>()
    }

    fn init_locals(&self) -> Vec<ast::Dynamic<'ctx>> {
        let mut locals: Vec<ast::Dynamic<'ctx>> = Vec::new();

        for param in self.z3_params.iter() {
            locals.push(param.clone());
        }

        locals.reserve(self.local_types.len());

        for local_type in &self.local_types {
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
        locals
    }
    // TODO(taegyunkim): Add test for each case.
    pub fn convert(&self, instrs: &[Instruction]) -> ast::Dynamic<'ctx> {
        let mut locals: Vec<ast::Dynamic<'ctx>> = self.init_locals();
        let mut stack: ValueStack<'ctx> = ValueStack::new();

        for instr in instrs {
            match instr {
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
                    locals[*idx as usize] = val;
                }
                Instruction::I32Const(c) => {
                    let val = ast::BV::from_i64(&self.ctx, *c as i64, 32);
                    stack.push(val);
                }
                _ => self.convert_op (&mut stack, instr),
            }
        }
        match self.func_type.return_type() {
            Some(_) => stack.pop(),
            None => panic!("Doens't support void functions."),
        }
    }

fn convert_op (&self, stack: &mut ValueStack<'ctx>, instr: &Instruction) {
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
            // NOTE(taegyunkim): The WASM spec tests performs shifts by modulo 32, ditto
            // all shift and rotate instructions.
            let shift_cnt = rhs.bvand(&ast::BV::from_i64(&self.ctx, 31, 32));
            let res = lhs.bvshl(&shift_cnt);
            stack.push(res);
        }
        Instruction::I32ShrS => {
            // NOTE(taegyunkim): sign-replicating (arithmetic) shift right.
            let (lhs, rhs) = stack.pop_pair_as::<ast::BV<'ctx>>();
            let shift_cnt = rhs.bvand(&ast::BV::from_i64(&self.ctx, 31, 32));
            let res = lhs.bvashr(&shift_cnt);
            stack.push(res);
        }
        Instruction::I32ShrU => {
            // NOTE(taegyunkim): zero-replicating (logical) shift right.
            let (lhs, rhs) = stack.pop_pair_as::<ast::BV<'ctx>>();
            let shift_cnt = rhs.bvand(&ast::BV::from_i64(&self.ctx, 31, 32));
            let res = lhs.bvlshr(&shift_cnt);
            stack.push(res);
        }
        Instruction::I32Rotl => {
            let (lhs, rhs) = stack.pop_pair_as::<ast::BV<'ctx>>();
            let rotate_cnt = rhs.bvand(&ast::BV::from_i64(&self.ctx, 31, 32));
            let res = lhs.bvrotl(&rotate_cnt);
            stack.push(res);
        }
        Instruction::I32Rotr => {
            let (lhs, rhs) = stack.pop_pair_as::<ast::BV<'ctx>>();
            let rotate_cnt = rhs.bvand(&ast::BV::from_i64(&self.ctx, 31, 32));
            let res = lhs.bvrotr(&rotate_cnt);
            stack.push(res);
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
    };

}
    pub fn convert_for_synthesis(&self, 
                                 instrs: &[Instruction],
                                 const_holes: &mut HashMap<usize, Box<ast::BV<'ctx>>>,
                                 local_holes: &mut HashMap<usize, Box<ast::BV<'ctx>>>) -> ast::Dynamic<'ctx> {

        let locals: Vec<ast::Dynamic<'ctx>> = self.init_locals();
        let mut stack: ValueStack<'ctx> = ValueStack::new();
        let mut hole_idx: usize = 0;
        let mut z3_locals = z3::ast::Array::new_const(&self.ctx,
            "locals",
            &z3::Sort::bitvector(&self.ctx,32),
            &z3::Sort::bitvector(&self.ctx,32));
        for (i, loc) in locals.iter().enumerate() {
            z3_locals = z3_locals
                .store(&ast::BV::from_i64(&self.ctx, i as i64, 32)
                .into(),
                &loc.clone());
        }
        for instr in instrs {
            match instr {
                // local variable ops
                Instruction::GetLocal(idx) => {
                    let hole = z3::ast::BV::new_const(&self.ctx, 
                        format!("c{}", hole_idx).as_str(), 
                        32);
                    let z3_val = z3_locals.select(&ast::Dynamic::from_ast(&hole.clone()));
                    match z3_val.as_bv() {
                        Some(x) => {
                            local_holes.insert(hole_idx, 
                                Box::new(hole));
                            stack.push(x)
                        },
                        _ => {
                            println!("GetLocal couldn't be synthesized");
                            let val = &locals[*idx as usize];
                            stack.push(val.clone())
                        },
                    }
                }
                Instruction::SetLocal(_) => {
                    let hole = z3::ast::BV::new_const(&self.ctx, 
                        format!("c{}", hole_idx).as_str(), 
                        32);
                    let val = stack.pop();
                    z3_locals.store(&hole.clone().into(), &val);
                    local_holes.insert(hole_idx, 
                        Box::new(hole));

                }
                Instruction::TeeLocal(_) => {
                    let hole = z3::ast::BV::new_const(&self.ctx, 
                        format!("c{}", hole_idx).as_str(), 
                        32);
                    let val = stack.pop();
                    stack.push(val.clone());
                    z3_locals.store(&ast::Dynamic::from_ast(&hole), 
                        &val.clone());
                    local_holes.insert(hole_idx, 
                        Box::new(hole));
                }
                Instruction::I32Const(_) => {
                    let hole = z3::ast::BV::new_const(&self.ctx, 
                        format!("c{}", hole_idx).as_str(), 
                        32);
                    const_holes.insert(hole_idx, 
                        Box::new(hole.clone()));
                    stack.push(hole);
                }
                _ => self.convert_op(&mut stack, instr),
            };
            hole_idx +=1;
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
        let converter = Converter::new(ctx, func_type, spec.locals());
        let spec_f = converter.convert(spec.code().elements());
        Self {
            ctx,
            converter,
            spec_f,
        }
    }

    pub fn synthesize (&self, 
                       instrs: &[Instruction],
                       min_const: i32,
                       max_const: i32) -> Option<Vec<Instruction>> {

        let locals: Vec<ast::Dynamic<'ctx>> = self.converter.init_locals();
        let mut tmp_locals = Vec::<&ast::Dynamic<'ctx>>::with_capacity(locals.len());

        let mut new_instrs = Vec::<Instruction>::with_capacity(instrs.len());
        for l in locals.iter() {
            tmp_locals.push(l);
        }
//        tmp_locals.clone_from_slice(&locals[..]);
        new_instrs.clone_from_slice(&instrs[..]);
        let solver = Solver::new(&self.ctx);
        let mut params = z3::Params::new(&self.ctx);
        // NOTE: if correct programs return UNSAT, it could be
        // that the timeout is too low
        params.set_u32("timeout", 2000 as u32);

        solver.set_params(&params);

        let mut local_holes = HashMap::new();
        let mut const_holes = HashMap::new();
        let candidate_f = self.converter.convert_for_synthesis(instrs, 
            &mut const_holes, 
            &mut local_holes);

        solver.push();
        let const_ceiling = ast::BV::from_i64(&self.ctx, 
            max_const as i64, 32);
        let const_floor = ast::BV::from_i64(&self.ctx, 
            min_const as i64, 32);

        for (_, c) in const_holes.iter() {
            solver.assert(&c.bvsle(&const_ceiling));
            solver.assert(&c.bvsge(&const_floor));
        }

        let forall = z3::ast::forall_const(
            &self.ctx,
            &tmp_locals,
            &[],
            &self.spec_f._eq(&candidate_f).into())
        .as_bool()
        .unwrap(); 

        solver.assert(&forall);
        match solver.check() {
            z3::SatResult::Sat => {
                let model = solver.get_model();
                // index of instr and hole used for that instr
                for (i, c) in const_holes.iter() {
                    let value =
                        match model.eval(&**c) {
                            Some(x) => x.as_i64().unwrap() as i32,
                            _ => {
                                println!("Const hole var not found");
                                return None;
                            }
                        };
                    println!("Const hole: ({}, {})", *i, value);
                    new_instrs[*i] = Instruction::I32Const(value);
                }
                for (i, c) in local_holes.iter() {
                    let value = 
                        match model.eval(&**c) {
                            Some(x) => x.as_i64().unwrap(),
                            _ => return None,
                        };
                    new_instrs[*i] = match instrs[*i] {
                        Instruction::GetLocal(_) => {
                            println!("get hole ({}, {})", i, value);
                            if value as usize >= locals.len() {
                                return None;
                            }
                            Instruction::GetLocal(value as u32)
                        }
                        Instruction::SetLocal(_) => {
                            println!("set hole ({}, {})", i, value);
                            if value as usize >= locals.len() {
                                return None;
                            }
                            Instruction::SetLocal(value as u32)
                        }
                        Instruction::TeeLocal(_) => {
                            println!("tee hole ({}, {})", i, value);
                            if value as usize >= locals.len() {
                                return None;
                            }
                            Instruction::TeeLocal(value as u32)
                        }
                        _ => panic!("Hole used incorrectly"),
                    };
                }
                Some(new_instrs)
            }
            _ => None
        }
    }
    pub fn verify(&self, instrs: &[Instruction]) -> VerifyResult {
        let candidate_f = self.converter.convert(instrs);
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
        assert_eq!(
            solver.verify(candidate_func_body.code().elements()),
            VerifyResult::Verified
        );
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
        assert_eq!(
            solver.verify(candidate_func_body.code().elements()),
            VerifyResult::Verified
        );
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
        let result = solver.verify(candidate_func_body.code().elements());
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
    // A test case that discvered bug in shift and rotate instructions.
    fn shift_verify_test() {
        let spec_module: parity_wasm::elements::Module = wat2module(
            r#"(module
                (type $t0 (func (param i32) (result i32)))
                (func $p3 (type $t0) (param $p0 i32) (result i32)
                  i32.const 0
                  local.get 0
                  i32.sub
                  local.get 0
                  i32.and)
                (export "p3" (func $p3)))"#,
        );
        let (spec_func_type, spec_func_body) = parity_wasm_utils::func_by_name(&spec_module, "p3");
        let candidate_module: parity_wasm::elements::Module = wat2module(
            r#"(module
                (type $t0 (func (param i32) (result i32)))
                (func $candidate (type $t0) (param $p0 i32) (result i32)
                  i32.const 1
                  local.get 0
                  i32.ctz
                  i32.shl)
                (export "candidate" (func $candidate)))"#,
        );
        let (candidate_func_type, candidate_func_body) =
            parity_wasm_utils::func_by_name(&candidate_module, "candidate");
        assert_eq!(spec_func_type, candidate_func_type);

        let cfg = z3::Config::new();
        let ctx = z3::Context::new(&cfg);

        let solver = Z3Solver::new(&ctx, spec_func_type, spec_func_body);
        let result = solver.verify(candidate_func_body.code().elements());
        assert_matches!(result, VerifyResult::CounterExample(_));
        if let VerifyResult::CounterExample(cex_vec) = result {
            let cex_vec = to_wasmi_values(cex_vec);
            assert_eq!(cex_vec, vec![wasmi::RuntimeValue::I32(0)]);

            let spec_instance = instantiate(spec_module);
            let spec_output = spec_instance
                .invoke_export("p3", &cex_vec, &mut wasmi::NopExternals)
                .unwrap();

            let candidate_instance = instantiate(candidate_module);
            let candidate_output = candidate_instance
                .invoke_export("candidate", &cex_vec, &mut wasmi::NopExternals)
                .unwrap();
            assert_ne!(spec_output, candidate_output);
            assert_eq!(spec_output, Some(wasmi::RuntimeValue::I32(0)));
            assert_eq!(candidate_output, Some(wasmi::RuntimeValue::I32(1)));
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
