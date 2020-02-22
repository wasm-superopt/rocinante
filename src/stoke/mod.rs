use crate::exec::{Interpreter, InterpreterKind};
use crate::{debug, exec, parity_wasm_utils, solver, Algorithm};
use parity_wasm::elements::{Internal, Module};
use rand::distributions::{Bernoulli, Distribution};
use rand::Rng;

use self::transform::*;
pub mod transform;
pub mod whitelist;
pub use self::candidate::*;
mod candidate;

pub struct Superoptimizer {
    algorithm: Algorithm,
    interpreter_kind: InterpreterKind,
    count_stack_off: bool,
    module: Module,
}

impl Superoptimizer {
    pub fn new(
        algorithm: Algorithm,
        interpreter_kind: InterpreterKind,
        count_stack_off: bool,
        module: Module,
    ) -> Self {
        Superoptimizer {
            algorithm,
            interpreter_kind,
            count_stack_off,
            module,
        }
    }

    pub fn run(&self) {}

    pub fn eval_candidate(&self, interpreter: &dyn Interpreter, candidate: &mut Candidate) -> u32 {
        if !self.count_stack_off && candidate.stack_cnt() != interpreter.return_type_len() as i32 {
            return interpreter.score_invalid();
        }
        let binary = candidate.get_binary();
        interpreter.eval_test_cases(&binary)
    }

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

                let mut candidate_func = Candidate::new(
                    func_type,
                    func_body.locals(),
                    func_body.code().elements().len(),
                    constants.clone(),
                );

                let mut curr_cost = self.eval_candidate(interpreter.as_ref(), &mut candidate_func);
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
                    let new_cost = self.eval_candidate(interpreter.as_ref(), &mut candidate_func);

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
