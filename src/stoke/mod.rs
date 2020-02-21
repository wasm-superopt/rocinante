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
                    func_body.locals(),
                    func_body.code().elements().len(),
                    constants.clone(),
                );

                let mut curr_cost = interpreter.eval_test_cases(&mut candidate_func);
                loop {
                    if curr_cost == 0 {
                        let module = candidate_func.to_module();
                        debug::print_functions(&module);
                        match z3solver.verify(&candidate_func.to_func_body()) {
                            solver::VerifyResult::Verified => {
                                println!("Verified.");
                                break;
                            }
                            solver::VerifyResult::CounterExample(values) => {
                                interpreter.add_test_case(&values);
                                println!("Added a new test case {:?}", values);
                                // Verifier finds one counterexample for now, so we update the
                                // cost to be the number of bits for return value type.
                                curr_cost = interpreter.return_bit_width();
                            }
                        }
                    }

                    let transform = rng.gen::<Transform>();
                    let transform_info = transform.operate(rng, &mut candidate_func);
                    let new_cost = interpreter.eval_test_cases(&mut candidate_func);

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
    stack_cnt: i32,
    constants: Vec<i32>,
    /// This field contains WASM binary generated from above func_type, with function name
    /// 'candidate'. It is initialized once when this struct is initialized and reused to avoid
    /// multiple conversions to binary format which are costly.
    binary: Vec<u8>,
    binary_len: usize,
}

impl CandidateFunc {
    pub fn new(
        func_type: &FunctionType,
        locals: &[Local],
        len: usize,
        constants: Vec<i32>,
    ) -> Self {
        let mut binary = parity_wasm_utils::build_module(
            "candidate",
            &func_type,
            FuncBody::new(vec![], Instructions::new(vec![])),
        )
        .to_bytes()
        .unwrap();

        // The last four bytes represent the code section of WASM, and always have 3, 1, 1, 0,
        // The value 3 represents the length of the sequence of bytes that follows, and the first
        // 1 means that there is one function. The last two bytes actually represent the function,
        // which can be translated to Nop, and Unreachable.
        binary = binary[0..binary.len() - 4].to_vec();
        // Reserve space ahead to avoid expensive memory operations during search.
        binary.reserve(2 + len);
        let binary_len = binary.len();

        // Keep track of the local types of the spec.
        let mut local_types = Vec::new();
        for l in locals {
            for _ in 0..l.count() {
                local_types.push(l.value_type());
            }
        }

        Self {
            func_type: func_type.clone(),
            local_types,
            // NOTE(taegyunkim): The original spec instruction list has an END instruction at the
            // end, so subtract one for that. We assume that we want to synthesize a function that
            // simply returns a value without any control flow.
            instrs: vec![Instruction::Nop; len - 1],
            stack_cnt: 0,
            constants,
            binary,
            binary_len,
        }
    }

    pub fn inc_stack_cnt(&mut self, n: i32) {
        self.stack_cnt += n;
    }

    pub fn dec_stack_cnt(&mut self, n: i32) {
        self.stack_cnt -= n;
    }

    pub fn stack_cnt(&self) -> i32 {
        self.stack_cnt
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
            &self.local_types[i - self.func_type.params().len()]
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
        // TODO(taegyunkim): For candidate programs that don't use locals, this can be removed.
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
        // TODO(taegyunkim): Avoid conversion to FuncBody and convert instruction list to binary.
        let func_binary = serialize::<FuncBody>(self.to_func_body()).unwrap();

        self.binary.truncate(self.binary_len);
        self.binary.extend(&[func_binary.len() as u8 + 1, 1]);
        self.binary.extend(func_binary);

        // TODO(taegyunkim): Add a test.
        //assert_eq!(self.binary, self.to_module().to_bytes().unwrap());
        &self.binary
    }
}
