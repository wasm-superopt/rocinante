use crate::exec::{Interpreter, InterpreterKind};
use crate::{exec, parity_wasm_utils, solver};
use parity_wasm::elements::{Instruction, Internal, Module};
use rand::distributions::{Bernoulli, Distribution};
use rand::Rng;
use std::sync::mpsc::channel;

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
    compute_budget: chrono::Duration,
    run_synthesis_only: bool,
    constants: Vec<i32>,
    beta: f64,
}

// TODO(taegyunkim): Use structopt https://docs.rs/structopt/0.3.9/structopt/index.html
impl SuperoptimizerOptions {
    pub fn new(
        algorithm: Algorithm,
        interpreter_kind: InterpreterKind,
        enforce_stack_check: bool,
        compute_budget: chrono::Duration,
        run_synthesis_only: bool,
        constants: Vec<i32>,
        beta: f64,
    ) -> Self {
        Self {
            algorithm,
            interpreter_kind,
            enforce_stack_check,
            compute_budget,
            run_synthesis_only,
            constants,
            beta,
        }
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
enum Mode {
    Synthesis,
    Optimization,
}

pub struct Superoptimizer {
    spec: Vec<u8>,
    options: SuperoptimizerOptions,
}

impl Superoptimizer {
    pub fn new(spec: Vec<u8>, options: SuperoptimizerOptions) -> Self {
        Superoptimizer { spec, options }
    }

    pub fn run(&self) {
        let module = Module::from_bytes(&self.spec).unwrap();

        // TODO(taegyunkim): Use num_cpus crate to appropriately set the number of workers.
        let num_workers = 1;
        let mut candidates: Vec<Candidate> = Vec::with_capacity(num_workers);

        let export_section = module
            .export_section()
            .expect("Module doesn't have export section.");

        for export_entry in export_section.entries() {
            if let Internal::Function(_idx) = export_entry.internal() {
                let func_name = export_entry.field();

                let (func_type, func_body) = parity_wasm_utils::func_by_name(&module, func_name);

                // TODO(taegyunkim): Parallel processing.
                for _ in 0..num_workers {
                    // NOTE(taegyunkim): Interpreter is not thread safe.
                    let mut interpreter =
                        exec::get_interpreter(self.options.interpreter_kind, &self.spec, func_name);

                    let mut candidate =
                        Candidate::new(func_type, func_body, self.options.constants.clone());

                    if self.do_run(Mode::Synthesis, interpreter.as_mut(), &mut candidate) {
                        candidate.strip_nops();
                        candidates.push(candidate.clone());

                        if self.options.run_synthesis_only {
                            continue;
                        }

                        if self.do_run(Mode::Optimization, interpreter.as_mut(), &mut candidate) {
                            candidate.strip_nops();
                            candidates.push(candidate);
                        }
                    }
                }
            }
        }

        self.rank(&candidates);
    }

    fn eval_candidate(
        &self,
        mode: Mode,
        interpreter: &dyn Interpreter,
        candidate: &mut Candidate,
    ) -> u32 {
        let mut cost = if self.options.enforce_stack_check {
            match candidate.check_stack() {
                StackState::Valid => {
                    let binary = candidate.get_binary();
                    interpreter.eval_test_cases(&binary)
                }
                StackState::Invalid(cnt) => {
                    // If the program is invalid we penalize it the stack value count difference.
                    interpreter.score_invalid()
                        + (i32::abs(interpreter.return_type_len() as i32 - cnt) as u32 + 1)
                }
            }
        } else {
            let binary = candidate.get_binary();
            interpreter.eval_test_cases(&binary)
        };

        if mode == Mode::Optimization {
            cost += self.perf(candidate.get_func_body().code().elements());
        }

        cost
    }

    pub fn perf(&self, instrs: &[Instruction]) -> u32 {
        let mut cnt = 0;
        for instr in instrs {
            if *instr != Instruction::Nop {
                cnt += 1;
            }
        }
        cnt
    }

    pub fn rank(&self, candidates: &[Candidate]) {
        #[cfg(debug_assertions)]
        println!("Found {} programs", candidates.len());

        let _best = candidates
            .iter()
            .min_by(|a, b| self.perf(a.instrs()).cmp(&self.perf(b.instrs())))
            .unwrap();

        #[cfg(debug_assertions)]
        println!(
            "{}",
            wasmprinter::print_bytes(_best.to_module().to_bytes().unwrap()).unwrap()
        );
    }

    fn do_run(
        &self,
        mode: Mode,
        interpreter: &mut dyn Interpreter,
        candidate: &mut Candidate,
    ) -> bool {
        let func_type = candidate.spec_func_type();
        let func_body = candidate.get_spec_func_body();

        let cfg = z3::Config::new();
        let ctx = z3::Context::new(&cfg);
        let z3solver = solver::Z3Solver::new(&ctx, func_type, func_body);

        let mut rng = rand::thread_rng();

        let mut curr_cost = self.eval_candidate(mode, interpreter, candidate);

        let initial_cost = curr_cost;

        let timer = timer::Timer::new();
        let (tx, rx) = channel();

        // It's necessary to name this variable to trigger the callback.
        let _guard = timer.schedule_with_delay(self.options.compute_budget, move || {
            let _ = tx.send(());
        });

        println!("{}", curr_cost);

        loop {
            if (mode == Mode::Optimization && curr_cost < initial_cost)
                || (mode == Mode::Synthesis && curr_cost == 0)
            {
                match z3solver.verify(&candidate.get_func_body()) {
                    solver::VerifyResult::Verified => {
                        #[cfg(debug_assertions)]
                        {
                            println!(
                                "{}",
                                wasmprinter::print_bytes(candidate.get_binary()).unwrap()
                            );
                            println!("Verified.");
                        }
                        println!("0");
                        return true;
                    }
                    solver::VerifyResult::CounterExample(values) => {
                        #[cfg(debug_assertions)]
                        println!("Adding a new test case {:?}", values);
                        interpreter.add_test_case(values);
                        // Verifier finds one counterexample for now, so we update the
                        // cost to be the number of bits for return value type.
                        curr_cost = interpreter.return_bit_width();
                    }
                }
            }

            if rx.try_recv().is_ok() {
                #[cfg(debug_assertions)]
                println!("{:?} timed out", mode);
                break;
            }

            let transform = rng.gen::<Transform>();
            let transform_info = transform.operate(&mut rng, candidate);
            let new_cost = self.eval_candidate(mode, interpreter, candidate);

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
                        let p: f64 = (1.0 as f64).min(
                            (-self.options.beta * (new_cost as f64) / (curr_cost as f64)).exp(),
                        );
                        let d = Bernoulli::new(p).unwrap();
                        #[cfg(debug_assertions)]
                        println!("p: {}", p);
                        let accept = d.sample(&mut rng);
                        if !accept {
                            #[cfg(debug_assertions)]
                            println!("undoing...");
                            transform.undo(&transform_info, candidate);
                        } else {
                            #[cfg(debug_assertions)]
                            println!("accepted...");
                            curr_cost = new_cost;
                        }
                    }
                }
            }

            if curr_cost < interpreter.score_invalid() {
                println!("{}", curr_cost);
            }
        }

        false
    }
}
