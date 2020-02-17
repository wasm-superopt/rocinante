use self::transform::*;
use crate::exec::InterpreterKind;
use crate::{debug, exec, parity_wasm_utils, solver, Algorithm};
use parity_wasm::elements::serialize;
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
    interpreter_kind: InterpreterKind,
    module: Module,
}

impl Superoptimizer {
    pub fn new(algorithm: Algorithm, interpreter_kind: InterpreterKind, module: Module) -> Self {
        Superoptimizer {
            algorithm,
            interpreter_kind,
            module,
        }
    }

    pub fn run(&self) {}

    /// Finds a module that has functions equivalent to the functions in the given module.
    pub fn synthesize<R: Rng + ?Sized>(&self, rng: &mut R, constants: Vec<i32>) {
        let export_section = self
            .module
            .export_section()
            .expect("Module doesn't have export section.");

        for export_entry in export_section.entries() {
            if let Internal::Function(_idx) = export_entry.internal() {
                let func_name = export_entry.field();

                let mut interpreter = exec::get_interpreter(
                    self.interpreter_kind,
                    &self.module.clone().to_bytes().unwrap(),
                    func_name,
                );

                let (func_type, func_body) =
                    parity_wasm_utils::func_by_name(&self.module, func_name);

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
                let mut curr_cost = interpreter.eval_test_cases(&candidate_func.get_binary());
                loop {
                    if curr_cost == 0 {
                        match z3solver.verify(&candidate_func.to_func_body()) {
                            solver::VerifyResult::Verified => {
                                println!("Verified.");
                                let module = candidate_func.to_module();
                                debug::print_functions(&module);

                                break;
                            }
                            solver::VerifyResult::CounterExample(values) => {
                                if interpreter.add_test_case(&values) {
                                    println!("Added new test case {:?}", values);
                                }
                            }
                        }
                    }

                    let transform = rng.gen::<Transform>();
                    let transform_info = transform.operate(rng, &mut candidate_func);
                    let new_cost = interpreter.eval_test_cases(&candidate_func.get_binary());

                    #[cfg(debug_assertions)]
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
                                // Following computes min(1, exp(-0.4 * new_cost/ curr_cost))
                                // TODO(taegyunkim): Use parameter \beta instead of -0.4
                                let p: f64 = (1.0 as f64)
                                    .min((-0.2 * (new_cost as f64) / (curr_cost as f64)).exp());
                                let d = Bernoulli::new(p).unwrap();
                                #[cfg(debug_assertions)]
                                println!("p: {}", p);
                                let accept = d.sample(rng);
                                if !accept {
                                    #[cfg(debug_assertions)]
                                    println!("undoing...");
                                    transform.undo(&transform_info, &mut candidate_func);
                                } else {
                                    #[cfg(debug_assertions)]
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
    binary: Vec<u8>,
    binary_len: usize,
}

impl CandidateFunc {
    pub fn new(func_type: &FunctionType, len: usize, constants: Vec<i32>) -> Self {
        let mut binary = parity_wasm_utils::build_module(
            "candidate",
            &func_type,
            FuncBody::new(vec![], Instructions::new(vec![])),
        )
        .to_bytes()
        .unwrap();

        binary = binary[0..binary.len() - 4].to_vec();
        binary.reserve(len);
        let binary_len = binary.len();

        Self {
            func_type: func_type.clone(),
            local_types: Vec::new(),
            // NOTE(taegyunkim): The original spec instruction list has an END instruction at the
            // end, so subtract one for that. We assume that we want to synthesize a function that
            // simply returns a value without any control flow.
            instrs: vec![Instruction::Nop; len - 1],
            constants,
            binary,
            binary_len,
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

        // NOTE(taegyunkim): As commented in the constructor we need to append an END instruction,
        // to make this a valid function representation.
        let mut instrs = self.instrs.clone();
        instrs.push(Instruction::End);

        FuncBody::new(locals, Instructions::new(instrs))
    }

    pub fn to_module(&self) -> Module {
        parity_wasm_utils::build_module("candidate", &self.func_type, self.to_func_body())
    }

    pub fn get_binary(&mut self) -> &[u8] {
        let func_binary = serialize::<FuncBody>(self.to_func_body()).unwrap();
        let mut ptr = self.binary.as_mut_ptr();
        unsafe {
            ptr = ptr.add(self.binary_len);
            let func_binary_len = func_binary.len();
            *ptr = func_binary_len as u8 + 1;
            ptr = ptr.add(1);
            *ptr = 1;
            ptr = ptr.add(1);

            for (i, item) in func_binary.into_iter().enumerate() {
                *ptr.add(i) = item;
            }

            self.binary.set_len(self.binary_len + 2 + func_binary_len);
        }

        //assert_eq!(self.binary, self.to_module().to_bytes().unwrap());
        &self.binary
    }
}
