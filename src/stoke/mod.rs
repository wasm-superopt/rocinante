use self::transform::*;
use crate::{exec, parity_wasm_utils, solver};
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
    pub fn synthesize<R: Rng + ?Sized>(&self, rng: &mut R, constants: Vec<i32>) {
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

                let candidate_func = CandidateFunc::new(func_type, constants.clone());
                let module = candidate_func.to_module();
                let curr_cost = exec::eval_test_cases(module, &test_cases);
                loop {
                    if curr_cost == 0 {
                        match z3solver.verify(&candidate_func.to_func_body()) {
                            solver::VerifyResult::Verified => {
                                println!("Verified.");
                                break;
                            }
                            solver::VerifyResult::CounterExample(_) => {
                                // TODO(taegyunkim): Add input, output pair to
                                // the test casese.
                            }
                        }
                    }

                    let _transform = rng.gen::<Transform>();
                }
            }
        }
    }
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

    pub fn get_rand_instr<R: Rng + ?Sized>(&self, rng: &mut R) -> (usize, Instruction) {
        let indices = rand::seq::index::sample(rng, self.instrs.len(), 1);
        (indices.index(0), self.instrs[indices.index(0)].clone())
    }

    pub fn get_equiv_local_idx<R: Rng + ?Sized>(&self, rng: &mut R, i: u32) -> u32 {
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

    pub fn sample_local_idx<R: Rng + ?Sized>(&self, rng: &mut R) -> u32 {
        rng.gen_range(0, self.func_type.params().len() + self.local_types.len()) as u32
    }

    pub fn sample_i32<R: Rng + ?Sized>(&self, rng: &mut R) -> i32 {
        *self.constants.choose(rng).unwrap()
    }

    pub fn instrs(&self) -> &[Instruction] {
        &self.instrs
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
