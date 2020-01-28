use crate::{debug, exec, parity_wasm_utils, solver};
use parity_wasm::elements::{FuncBody, FunctionType, Instruction, Instructions, Internal, Module};
use rand::distributions::{Distribution, Standard};
use rand::seq::SliceRandom;
use rand::Rng;

#[allow(dead_code)]
pub struct Superoptimizer {
    module: Module,
}

impl Superoptimizer {
    pub fn new(module: Module) -> Self {
        Superoptimizer { module }
    }

    pub fn run(&self) {}

    /// Finds a module that has functions equivalent to the functions in the given module.
    pub fn synthesize(&self, rng: &mut impl Rng) {
        // Module in wasmi, WASM interpreter. Instantiate this here and pass
        // down to exec module functions to avoid re-instantiation.
        let wasmi_module = wasmi::Module::from_parity_wasm_module(self.module.clone())
            .expect("Failed to load parity-wasm Module.");
        let instance = wasmi::ModuleInstance::new(&wasmi_module, &wasmi::ImportsBuilder::default())
            .expect("Failed to instantiate wasm module.")
            .assert_no_start();

        let export_section = self
            .module
            .export_section()
            .expect("Module doesn't have export section.");

        for export_entry in export_section.entries() {
            if let Internal::Function(_idx) = export_entry.internal() {
                let func_name = export_entry.field();

                let test_cases = exec::generate_test_cases(rng, &instance, func_name);
                // let _generator = Generator::new(&func_type);
                let (func_type, func_body) =
                    parity_wasm_utils::func_by_name(&self.module, func_name);

                let cfg = z3::Config::new();
                let ctx = z3::Context::new(&cfg);
                let z3solver = solver::Z3Solver::new(&ctx, func_type, func_body);
                let mut generator = Generator::new(func_type);

                loop {
                    let module = generator.module();
                    if exec::eval_test_cases(module.clone(), &test_cases) > 0 {
                        generator.do_transform(rng);
                        continue;
                    }
                    match z3solver.verify(generator.get_candidate_func()) {
                        solver::VerifyResult::Verified => {
                            // collect the function from generator
                            debug::print_functions(&module);
                            break;
                        }
                        solver::VerifyResult::CounterExample(_) => {
                            // Add input, output pair to the test cases.
                            generator.do_transform(rng)
                        }
                    }
                }
            }
        }
    }
}

#[derive(Debug)]
pub enum Transform {
    Opcode,
    Operand,
    Swap,
    Instruction,
}

impl Distribution<Transform> for Standard {
    fn sample<R: Rng + ?Sized>(&self, rng: &mut R) -> Transform {
        match rng.gen_range(0, 4) {
            0 => Transform::Opcode,
            1 => Transform::Operand,
            2 => Transform::Swap,
            _ => Transform::Instruction,
        }
    }
}

#[derive(Debug, PartialEq, Eq, Copy, Clone)]
pub enum WhitelistedInstruction {
    I32Add,
    I32Sub,
    I32Mul,
    I32DivS,
    I32DivU,
    I32RemS,
    I32RemU,
    I32And,
    I32Or,
    I32Xor,
    I32Shl,
    I32ShrS,
    I32ShrU,
    I32Rotl,
    I32Rotr,
    GetLocal(u32),
    SetLocal(u32),
    TeeLocal(u32),
    End,
    Nop,
}

impl WhitelistedInstruction {}

impl std::fmt::Display for WhitelistedInstruction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match *self {
            WhitelistedInstruction::I32Add => write!(f, "i32.add"),
            WhitelistedInstruction::I32Sub => write!(f, "i32.sub"),
            WhitelistedInstruction::I32Mul => write!(f, "i32.mul"),
            WhitelistedInstruction::I32DivS => write!(f, "i32.divs"),
            WhitelistedInstruction::I32DivU => write!(f, "i32.divu"),
            WhitelistedInstruction::I32RemS => write!(f, "i32.rems"),
            WhitelistedInstruction::I32RemU => write!(f, "i32.remu"),
            WhitelistedInstruction::I32And => write!(f, "i32.and"),
            WhitelistedInstruction::I32Or => write!(f, "i32.or"),
            WhitelistedInstruction::I32Xor => write!(f, "i32.xor"),
            WhitelistedInstruction::I32Shl => write!(f, "i32.shl"),
            WhitelistedInstruction::I32ShrS => write!(f, "i32.shrs"),
            WhitelistedInstruction::I32ShrU => write!(f, "i32.shru"),
            WhitelistedInstruction::I32Rotl => write!(f, "i32.rotl"),
            WhitelistedInstruction::I32Rotr => write!(f, "i32.rotr"),
            WhitelistedInstruction::GetLocal(i) => write!(f, "get_local {}", i),
            WhitelistedInstruction::SetLocal(i) => write!(f, "set_local {}", i),
            WhitelistedInstruction::TeeLocal(i) => write!(f, "tee_local {}", i),
            WhitelistedInstruction::End => write!(f, "end"),
            WhitelistedInstruction::Nop => write!(f, "nop"),
        }
    }
}

impl From<Instruction> for WhitelistedInstruction {
    fn from(instr: Instruction) -> Self {
        match instr {
            Instruction::I32Add => WhitelistedInstruction::I32Add,
            Instruction::I32Sub => WhitelistedInstruction::I32Sub,
            Instruction::I32Mul => WhitelistedInstruction::I32Mul,
            Instruction::I32DivS => WhitelistedInstruction::I32DivS,
            Instruction::I32DivU => WhitelistedInstruction::I32DivU,
            Instruction::I32RemS => WhitelistedInstruction::I32RemS,
            Instruction::I32RemU => WhitelistedInstruction::I32RemU,
            Instruction::I32And => WhitelistedInstruction::I32And,
            Instruction::I32Or => WhitelistedInstruction::I32Or,
            Instruction::I32Xor => WhitelistedInstruction::I32Xor,
            Instruction::I32Shl => WhitelistedInstruction::I32Shl,
            Instruction::I32ShrS => WhitelistedInstruction::I32ShrS,
            Instruction::I32ShrU => WhitelistedInstruction::I32ShrU,
            Instruction::I32Rotl => WhitelistedInstruction::I32Rotl,
            Instruction::I32Rotr => WhitelistedInstruction::I32Rotr,
            Instruction::GetLocal(i) => WhitelistedInstruction::GetLocal(i),
            Instruction::SetLocal(i) => WhitelistedInstruction::SetLocal(i),
            Instruction::TeeLocal(i) => WhitelistedInstruction::TeeLocal(i),
            Instruction::End => WhitelistedInstruction::End,
            Instruction::Nop => WhitelistedInstruction::Nop,
            _ => panic!("{} not implemented", instr),
        }
    }
}

impl Into<Instruction> for WhitelistedInstruction {
    fn into(self) -> Instruction {
        match self {
            WhitelistedInstruction::I32Add => Instruction::I32Add,
            WhitelistedInstruction::I32Sub => Instruction::I32Sub,
            WhitelistedInstruction::I32Mul => Instruction::I32Mul,
            WhitelistedInstruction::I32DivS => Instruction::I32DivS,
            WhitelistedInstruction::I32DivU => Instruction::I32DivU,
            WhitelistedInstruction::I32RemS => Instruction::I32RemS,
            WhitelistedInstruction::I32RemU => Instruction::I32RemU,
            WhitelistedInstruction::I32And => Instruction::I32And,
            WhitelistedInstruction::I32Or => Instruction::I32Or,
            WhitelistedInstruction::I32Xor => Instruction::I32Xor,
            WhitelistedInstruction::I32Shl => Instruction::I32Shl,
            WhitelistedInstruction::I32ShrS => Instruction::I32ShrS,
            WhitelistedInstruction::I32ShrU => Instruction::I32ShrU,
            WhitelistedInstruction::I32Rotl => Instruction::I32Rotl,
            WhitelistedInstruction::I32Rotr => Instruction::I32Rotr,
            WhitelistedInstruction::GetLocal(i) => Instruction::GetLocal(i),
            WhitelistedInstruction::SetLocal(i) => Instruction::SetLocal(i),
            WhitelistedInstruction::TeeLocal(i) => Instruction::TeeLocal(i),
            WhitelistedInstruction::End => Instruction::End,
            WhitelistedInstruction::Nop => Instruction::Nop,
        }
    }
}

impl Distribution<WhitelistedInstruction> for Standard {
    fn sample<R: Rng + ?Sized>(&self, rng: &mut R) -> WhitelistedInstruction {
        match rng.gen_range(0, 20) {
            0 => WhitelistedInstruction::I32Add,
            1 => WhitelistedInstruction::I32Sub,
            2 => WhitelistedInstruction::I32Mul,
            3 => WhitelistedInstruction::I32DivS,
            4 => WhitelistedInstruction::I32DivU,
            5 => WhitelistedInstruction::I32RemS,
            6 => WhitelistedInstruction::I32RemU,
            7 => WhitelistedInstruction::I32And,
            8 => WhitelistedInstruction::I32Or,
            9 => WhitelistedInstruction::I32Xor,
            10 => WhitelistedInstruction::I32Shl,
            11 => WhitelistedInstruction::I32ShrS,
            12 => WhitelistedInstruction::I32ShrU,
            13 => WhitelistedInstruction::I32Rotl,
            14 => WhitelistedInstruction::I32Rotr,
            15 => WhitelistedInstruction::GetLocal(0),
            16 => WhitelistedInstruction::SetLocal(0),
            17 => WhitelistedInstruction::TeeLocal(0),
            18 => WhitelistedInstruction::End,
            _ => WhitelistedInstruction::Nop,
        }
    }
}

pub struct TransformPools {}

const I32BINOP: [Instruction; 15] = [
    Instruction::I32Add,
    Instruction::I32Sub,
    Instruction::I32Mul,
    Instruction::I32DivS,
    Instruction::I32DivU,
    Instruction::I32RemS,
    Instruction::I32RemU,
    Instruction::I32And,
    Instruction::I32Or,
    Instruction::I32Xor,
    Instruction::I32Shl,
    Instruction::I32ShrS,
    Instruction::I32ShrU,
    Instruction::I32Rotl,
    Instruction::I32Rotr,
];

const VAROP: [fn(n: u32) -> Instruction; 3] = [
    Instruction::GetLocal,
    Instruction::SetLocal,
    Instruction::TeeLocal,
    // Instruction::GetGlobal,
    // Instruction::SetGlobal,
];

impl TransformPools {
    fn new() -> Self {
        Self {}
    }
    fn get_equiv<R: Rng + ?Sized>(&self, rng: &mut R, instr: &Instruction) -> Instruction {
        // Make sure this instruction is whitelisted.
        let _: WhitelistedInstruction = instr.clone().into();

        match instr {
            _ if I32BINOP.contains(instr) => I32BINOP.choose(rng).unwrap().clone(),
            Instruction::GetLocal(i) | Instruction::SetLocal(i) | Instruction::TeeLocal(i) => {
                (*VAROP.choose(rng).unwrap())(*i)
            }
            Instruction::End => Instruction::End,
            Instruction::Nop => Instruction::Nop,
            _ => {
                panic!("not implemented.");
            }
        }
    }
}

pub struct Generator {
    pools: TransformPools,
    func_type: FunctionType,
    func_body: FuncBody,
}

impl Generator {
    pub fn new(func_type: &FunctionType) -> Self {
        let func_body = FuncBody::new(vec![], Instructions::empty());
        Self {
            pools: TransformPools::new(),
            func_type: func_type.clone(),
            func_body,
        }
    }

    // fn get_local_types(&self, param_types: &[ValueType], locals: &[Local]) -> Vec<ValueType> {
    //     let mut types = param_types.to_vec();

    //     for local in locals {
    //         let count = local.count() as usize;
    //         types.reserve(count);
    //         let local_type = local.value_type();
    //         for _ in 0..count {
    //             types.push(local_type);
    //         }
    //     }

    //     types
    // }

    pub fn do_transform<R: Rng>(&mut self, rng: &mut R) {
        let transform: Transform = rng.gen::<Transform>();
        let instrs = self.func_body.code().elements();

        match transform {
            Transform::Opcode => {
                // Choose an instruction at random, and replace with a random,
                // equivalent one.
                let idx: usize = rng.gen_range(0, instrs.len());
                let chosen_instr = &instrs[idx];
                let new_instr = self.pools.get_equiv(rng, chosen_instr);
                let mut new_instrs = Vec::with_capacity(instrs.len());
                new_instrs.clone_from_slice(instrs);
                new_instrs[idx] = new_instr;
            }
            Transform::Operand => {
                // Select an instruction at random, and its operand is replaced by a
                // random operand drawn from an equivalence class of operands.
                let idx: usize = rng.gen_range(0, instrs.len());
                let chosen_instr = &instrs[idx];

                match chosen_instr {
                    Instruction::GetLocal(_)
                    | Instruction::SetLocal(_)
                    | Instruction::TeeLocal(_) => {
                        // Get index of local variable that has the same type,
                        // or index of a new local variable.
                    }
                    _ if I32BINOP.contains(chosen_instr) => {}
                    Instruction::End => {}
                    _ => {
                        panic!("Not implemented");
                    }
                }
            }
            Transform::Swap => {
                // Select two instructions from the set of original instructions
                // union with Nop, and swap
                let idx1 = rng.gen_range(0, instrs.len() + 1);
                let idx2 = rng.gen_range(0, instrs.len() + 1);
                let mut new_instrs = Vec::with_capacity(instrs.len());
                new_instrs.clone_from_slice(instrs);
                if idx1 < instrs.len() && idx2 < instrs.len() {
                    new_instrs[idx1] = instrs[idx2].clone();
                    new_instrs[idx2] = instrs[idx1].clone();
                } else if idx1 < instrs.len() && idx2 >= instrs.len() {
                    new_instrs[idx1] = Instruction::Nop;
                } else if idx1 >= instrs.len() && idx2 < instrs.len() {
                    new_instrs[idx2] = Instruction::Nop;
                } else {
                    // Do nothing
                }
            }
            Transform::Instruction => {
                let _idx = rng.gen_range(0, instrs.len());
                // Select an instruction, and replace with a random instruction,
                // with random operands.
            }
        }
    }

    pub fn module(&self) -> Module {
        parity_wasm_utils::build_module("candidate", &self.func_type, self.func_body.clone())
    }

    pub fn get_candidate_func(&self) -> &FuncBody {
        &self.func_body
    }
}
