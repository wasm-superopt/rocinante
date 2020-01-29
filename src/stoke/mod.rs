use crate::{debug, exec, parity_wasm_utils, solver};
use parity_wasm::elements::{
    FuncBody, FunctionType, Instruction, Instructions, Internal, Module, ValueType,
};
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
                    debug::print_functions(&module);
                    if exec::eval_test_cases(module.clone(), &test_cases) > 0 {
                        generator.do_transform(rng);
                        continue;
                    }
                    match z3solver.verify(generator.get_candidate_func()) {
                        solver::VerifyResult::Verified => {
                            // collect the function from generator
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
    I32Const(i32),
    GetLocal(u32),
    SetLocal(u32),
    TeeLocal(u32),
    End,
    Nop,
}

impl WhitelistedInstruction {
    pub fn sample<R: Rng + ?Sized>(
        rng: &mut R,
        param_types: &[ValueType],
        // TODO: Support increasing the number of locals.
        _local_types: &mut Vec<ValueType>,
    ) -> WhitelistedInstruction {
        match rng.gen_range(0, 21) {
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
            15 => {
                let operands = vec![-2, -1, 0, 1, 2];
                WhitelistedInstruction::I32Const(*operands.choose(rng).unwrap())
            }
            16 => {
                let idx = rng.gen_range(0, param_types.len()) as u32;
                WhitelistedInstruction::GetLocal(idx)
            }
            17 => {
                let idx = rng.gen_range(0, param_types.len()) as u32;
                WhitelistedInstruction::SetLocal(idx)
            }
            18 => {
                let idx = rng.gen_range(0, param_types.len()) as u32;
                WhitelistedInstruction::TeeLocal(idx)
            }
            19 => WhitelistedInstruction::End,
            _ => WhitelistedInstruction::Nop,
        }
    }
}

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
            WhitelistedInstruction::I32Const(i) => write!(f, "i32.const {}", i),
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
            Instruction::I32Const(i) => WhitelistedInstruction::I32Const(i),
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
            WhitelistedInstruction::I32Const(i) => Instruction::I32Const(i),
            WhitelistedInstruction::GetLocal(i) => Instruction::GetLocal(i),
            WhitelistedInstruction::SetLocal(i) => Instruction::SetLocal(i),
            WhitelistedInstruction::TeeLocal(i) => Instruction::TeeLocal(i),
            WhitelistedInstruction::End => Instruction::End,
            WhitelistedInstruction::Nop => Instruction::Nop,
        }
    }
}

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

pub struct Generator {
    func_type: FunctionType,
    func_body: FuncBody,
    local_types: Vec<ValueType>,
}

impl Generator {
    pub fn new(func_type: &FunctionType) -> Self {
        let instrs = vec![
            Instruction::GetLocal(0),
            Instruction::GetLocal(0),
            Instruction::I32Mul,
            Instruction::End,
        ];
        let func_body = FuncBody::new(vec![], Instructions::new(instrs));
        Self {
            func_type: func_type.clone(),
            func_body,
            local_types: Vec::new(),
        }
    }

    fn get_equiv<R: Rng + ?Sized>(&self, rng: &mut R, instr: &Instruction) -> Instruction {
        // Make sure this instruction is whitelisted.
        let _: WhitelistedInstruction = instr.clone().into();

        match instr {
            _ if I32BINOP.contains(instr) => I32BINOP.choose(rng).unwrap().clone(),
            Instruction::GetLocal(i) | Instruction::SetLocal(i) | Instruction::TeeLocal(i) => {
                (*VAROP.choose(rng).unwrap())(*i)
            }
            Instruction::I32Const(i) => Instruction::I32Const(*i),
            Instruction::End => Instruction::End,
            Instruction::Nop => Instruction::Nop,
            _ => {
                panic!("not implemented.");
            }
        }
    }

    fn get_equiv_idx<R: Rng + ?Sized>(&self, rng: &mut R, i: u32) -> u32 {
        let i = i as usize;
        let typ: &ValueType = if i < self.func_type.params().len() {
            &self.func_type.params()[i]
        } else if i < self.func_type.params().len() + self.local_types.len() {
            &self.local_types[i]
        } else {
            panic!("local index out of bounds: {}", i);
        };

        let mut equiv_indices = Vec::new();
        for (i, param_type) in self.func_type.params().iter().enumerate() {
            if param_type == typ {
                equiv_indices.push(i);
            }
        }

        for (i, local_type) in self.local_types.iter().enumerate() {
            if local_type == typ {
                equiv_indices.push(i + self.func_type.params().len());
            }
        }

        assert!(!equiv_indices.is_empty());

        *equiv_indices.choose(rng).unwrap() as u32
    }

    pub fn do_transform<R: Rng>(&mut self, rng: &mut R) {
        let transform: Transform = rng.gen::<Transform>();
        let instrs = self.func_body.code().elements();

        let mut new_instrs = instrs.to_vec();

        match transform {
            Transform::Opcode => {
                // Choose an instruction at random, and replace with a random,
                // equivalent one.
                let idx: usize = rng.gen_range(0, instrs.len());
                let chosen_instr = &instrs[idx];
                let new_instr = self.get_equiv(rng, chosen_instr);
                new_instrs[idx] = new_instr;
            }
            Transform::Operand => {
                // Select an instruction at random, and its operand is replaced by a
                // random operand drawn from an equivalence class of operands.
                let idx: usize = rng.gen_range(0, instrs.len());
                let chosen_instr = &instrs[idx];

                let new_instr = match chosen_instr {
                    Instruction::GetLocal(i) => Instruction::GetLocal(self.get_equiv_idx(rng, *i)),
                    Instruction::SetLocal(i) => Instruction::SetLocal(self.get_equiv_idx(rng, *i)),
                    Instruction::TeeLocal(i) => Instruction::SetLocal(self.get_equiv_idx(rng, *i)),
                    _ if I32BINOP.contains(chosen_instr) => chosen_instr.clone(),
                    Instruction::End => Instruction::End,
                    Instruction::Nop => Instruction::Nop,
                    Instruction::I32Const(_) => {
                        let operands = vec![-2, -1, 0, 1, 2];
                        Instruction::I32Const(*operands.choose(rng).unwrap())
                    }
                    _ => {
                        panic!("Not implemented");
                    }
                };
                new_instrs[idx] = new_instr;
            }
            Transform::Swap => {
                // Select two instructions from the set of original instructions
                // union with Nop, and swap
                let idx1 = rng.gen_range(0, instrs.len() + 1);
                let idx2 = rng.gen_range(0, instrs.len() + 1);
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
                let idx = rng.gen_range(0, instrs.len());
                let new_instr = WhitelistedInstruction::sample(
                    rng,
                    self.func_type.params(),
                    &mut self.local_types,
                );
                new_instrs[idx] = new_instr.into();
            }
        }

        self.func_body
            .code_mut()
            .elements_mut()
            .clone_from(&new_instrs);
    }

    pub fn module(&self) -> Module {
        parity_wasm_utils::build_module("candidate", &self.func_type, self.func_body.clone())
    }

    pub fn get_candidate_func(&self) -> &FuncBody {
        &self.func_body
    }
}
