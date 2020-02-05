use self::transform::*;
use crate::{debug, exec, parity_wasm_utils, solver, Algorithm};
use parity_wasm::elements::{
    FuncBody, FunctionType, Instruction, Instructions, Internal, Local, Module, ValueType,
};
use rand::distributions::{Bernoulli, Distribution};
use rand::seq::SliceRandom;
use rand::Rng;

pub mod transform;
pub mod whitelist;

pub struct Superoptimizer {
    algorithm: Algorithm,
    module: Module,
}

impl Superoptimizer {
    pub fn new(algorithm: Algorithm, module: Module) -> Self {
        Superoptimizer { algorithm, module }
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
                let return_type = func_type.return_type();

                // Check whether the spec contains only whitelisted instructions.
                whitelist::validate(func_body.code().elements());

                let cfg = z3::Config::new();
                let ctx = z3::Context::new(&cfg);
                let z3solver = solver::Z3Solver::new(&ctx, func_type, func_body);

                let mut candidate_func = CandidateFunc::new(
                    func_type,
                    func_body.code().elements().len(),
                    constants.clone(),
                );
                let mut module = candidate_func.to_module();
                let mut curr_cost = exec::eval_test_cases(&module, return_type, &test_cases);
                loop {
                    debug::print_functions(&module);

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

                    let transform = rng.gen::<Transform>();
                    let transform_info = transform.operate(rng, &mut candidate_func);

                    module = candidate_func.to_module();
                    let new_cost = exec::eval_test_cases(&module, return_type, &test_cases);

                    println!("curr_cost: {}, new_cost: {}", curr_cost, new_cost);
                    match self.algorithm {
                        Algorithm::Random => {
                            // Always accept transform.
                            curr_cost = new_cost;
                        }
                        Algorithm::Stoke => {
                            if new_cost < curr_cost {
                                // Accept this transform.
                                curr_cost = new_cost;
                            } else {
                                // Following computes min(1, exp(-1.0 * new_cost/ curr_cost))
                                // TODO(taegyunkim): Use parameter \beta instead of -1.0
                                let p: f64 = (1.0 as f64)
                                    .min((-1.0 * (new_cost as f64) / (curr_cost as f64)).exp());
                                let d = Bernoulli::new(p).unwrap();
                                println!("p: {}", p);
                                let accept = d.sample(rng);
                                if !accept {
                                    println!("undoing...");
                                    transform.undo(&transform_info, &mut candidate_func);
                                } else {
                                    println!("accepted...");
                                    curr_cost = new_cost;
                                }
                            }
                        }
                    }
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
    pub fn new(func_type: &FunctionType, len: usize, constants: Vec<i32>) -> Self {
        Self {
            func_type: func_type.clone(),
            local_types: Vec::new(),
            instrs: vec![Instruction::Nop; len],
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
