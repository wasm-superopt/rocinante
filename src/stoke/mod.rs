use self::transform::*;
use self::whitelist::*;
use crate::{debug, exec, parity_wasm_utils, solver};
use parity_wasm::elements::{
    FuncBody, FunctionType, Instruction, Instructions, Internal, Local, Module, ValueType,
};
use rand::seq::SliceRandom;
use rand::Rng;

pub mod transform;
pub mod whitelist;

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
    pub fn synthesize(&self, rng: &mut impl Rng, constants: Vec<i32>) {
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
                let (func_type, func_body) =
                    parity_wasm_utils::func_by_name(&self.module, func_name);

                // Check whether the spec contains only whitelisted instructions.
                whitelist::validate(func_body.code().elements());

                let cfg = z3::Config::new();
                let ctx = z3::Context::new(&cfg);
                let z3solver = solver::Z3Solver::new(&ctx, func_type, func_body);
                let mut generator = Generator::new(func_type, constants.clone());

                loop {
                    // TODO(taegyunkim): Implement undo of a transformation.
                    let module = generator.module();
                    debug::print_functions(&module);
                    if exec::eval_test_cases(module.clone(), &test_cases) > 0 {
                        generator.do_transform(rng);
                        continue;
                    }
                    match z3solver.verify(generator.get_candidate_func()) {
                        solver::VerifyResult::Verified => {
                            println!("Verified.");
                            // collect the function from generator
                            break;
                        }
                        solver::VerifyResult::CounterExample(_) => {
                            // TODO(taegyunkim): Add input, output pair to the test cases.
                            generator.do_transform(rng)
                        }
                    }
                }
            }
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
    // TOOD(taegyunkim): Support i64, f32, f64 constants.
    constants: Vec<i32>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CandidateFunc {
    func_type: FunctionType,
    local_types: Vec<ValueType>,
    instrs: Vec<Instruction>,
    constants: Vec<i32>,
}

impl CandidateFunc {
    pub fn new(func_type: &FunctionType, constants: Vec<i32>) -> Self {
        // TODO(taegyunkim): Generate a random program of length n.
        let instrs = vec![
            Instruction::GetLocal(0),
            Instruction::GetLocal(0),
            Instruction::I32Mul,
            Instruction::End,
        ];

        Self {
            func_type: func_type.clone(),
            local_types: Vec::new(),
            instrs,
            constants,
        }
    }

    pub fn get_rand_instr<R: Rng>(&self, rng: &mut R) -> (usize, Instruction) {
        let indices = rand::seq::index::sample(rng, self.instrs.len(), 1);
        (indices.index(0), self.instrs[indices.index(0)].clone())
    }

    pub fn get_equiv_idx<R: Rng>(&self, rng: &mut R, i: u32) -> u32 {
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

    pub fn sample_i32<R: Rng>(&self, rng: &mut R) -> i32 {
        *self.constants.choose(rng).unwrap()
    }

    pub fn instrs_mut(&mut self) -> &mut Vec<Instruction> {
        &mut self.instrs
    }

    pub fn to_func_body(&self) -> FuncBody {
        let locals: Vec<Local> = self
            .local_types
            .iter()
            .map(|typ| Local::new(1, *typ))
            .collect();

        FuncBody::new(locals, Instructions::new(self.instrs.clone()))
    }

    pub fn to_module(&self) -> Module {
        parity_wasm_utils::build_module("candidate", &self.func_type, self.to_func_body())
    }
}

impl Generator {
    pub fn new(func_type: &FunctionType, constants: Vec<i32>) -> Self {
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
            constants,
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

    fn sample_i32<R: Rng>(&self, rng: &mut R) -> i32 {
        *self.constants.choose(rng).unwrap()
    }

    pub fn do_transform<R: Rng>(&mut self, rng: &mut R) {
        let transform_kind: TransformKind = rng.gen::<TransformKind>();
        let instrs = self.func_body.code().elements();

        let mut new_instrs = instrs.to_vec();

        match transform_kind {
            TransformKind::Opcode => {
                // Choose an instruction at random, and replace with a random,
                // equivalent one.
                let idx: usize = rng.gen_range(0, instrs.len());
                let chosen_instr = &instrs[idx];
                let new_instr = self.get_equiv(rng, chosen_instr);
                new_instrs[idx] = new_instr;
            }
            TransformKind::Operand => {
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
                    Instruction::I32Const(_) => Instruction::I32Const(self.sample_i32(rng)),
                    _ => {
                        panic!("Not implemented");
                    }
                };
                new_instrs[idx] = new_instr;
            }
            TransformKind::Swap => {
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
            TransformKind::Instruction => {
                let idx = rng.gen_range(0, instrs.len());
                let new_instr = WhitelistedInstruction::sample(
                    rng,
                    self.func_type.params(),
                    &self.constants,
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
