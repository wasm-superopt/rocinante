use crate::exec::{Interpreter, InterpreterKind};
use crate::{exec, parity_wasm_utils, solver};
use parity_wasm::elements::{Internal, Module};
use rand::distributions::{Bernoulli, Distribution};
use rand::Rng;
use wasmprinter;

use self::transform::*;
pub mod transform;
pub mod whitelist;
pub use self::candidate::*;
mod candidate;

#[derive(Debug, Clone, Copy, PartialEq, Eq, EnumString)]
pub enum Algorithm {
    Random,
    Stoke,
}

pub struct SuperoptimizerOptions {
    algorithm: Algorithm,
    interpreter_kind: InterpreterKind,
    enforce_stack_check: bool,
}

impl SuperoptimizerOptions {
    pub fn new(
        algorithm: Algorithm,
        interpreter_kind: InterpreterKind,
        enforce_stack_check: bool,
    ) -> Self {
        Self {
            algorithm,
            interpreter_kind,
            enforce_stack_check,
        }
    }
}

pub struct Superoptimizer {
    spec: Vec<u8>,
    options: SuperoptimizerOptions,
}

impl Superoptimizer {
    pub fn new(spec: Vec<u8>, options: SuperoptimizerOptions) -> Self {
        Superoptimizer { spec, options }
    }

    pub fn run(&self) {}

    pub fn eval_candidate(&self, interpreter: &dyn Interpreter, candidate: &mut Candidate) -> u32 {
        if self.options.enforce_stack_check
            && candidate.stack_cnt() != interpreter.return_type_len() as i32
        {
            return interpreter.score_invalid();
        }
        let binary = candidate.get_binary();
        interpreter.eval_test_cases(&binary)
    }

    /// Finds a module that has functions equivalent to the functions in the given module.
    pub fn synthesize<R: Rng + ?Sized>(&self, rng: &mut R, constants: Vec<i32>) {
        let module = Module::from_bytes(&self.spec).expect("Failed to deserialize.");

        let export_section = module
            .export_section()
            .expect("Module doesn't have export section.");

        for export_entry in export_section.entries() {
            if let Internal::Function(_idx) = export_entry.internal() {
                let func_name = export_entry.field();

                let mut interpreter = exec::get_interpreter(
                    self.options.interpreter_kind,
                    &module.clone().to_bytes().unwrap(),
                    func_name,
                );

                let (func_type, func_body) = parity_wasm_utils::func_by_name(&module, func_name);

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
                        println!(
                            "{}",
                            wasmprinter::print_bytes(
                                candidate_func.to_module().to_bytes().unwrap()
                            )
                            .unwrap()
                        );
                        match z3solver.verify(&candidate_func.to_func_body()) {
                            solver::VerifyResult::Verified => {
                                println!("Verified.");
                                break;
                            }
                            solver::VerifyResult::CounterExample(values) => {
                                println!("Adding a new test case {:?}", values);
                                interpreter.add_test_case(values);
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
                    match self.options.algorithm {
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
