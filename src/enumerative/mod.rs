use crate::{exec, solver, stoke, stoke::Candidate, SuperoptimizerOpts};

pub fn run(spec: Vec<u8>, candidate: &mut Candidate, options: SuperoptimizerOpts, func_name: &str) {
    let mut interpreter = exec::get_interpreter(options.interpreter_kind, &spec, func_name);

    let spec_func_type = candidate.spec_func_type();
    let spec_func_body = candidate.get_spec_func_body();

    let cfg = z3::Config::new();
    let ctx = z3::Context::new(&cfg);
    let z3_solver = solver::Z3Solver::new(&ctx, spec_func_type, spec_func_body);

    // Set up timer and channels to send time outs.
    let timer = timer::Timer::new();
    let (tx, rx) = std::sync::mpsc::channel();
    let _guard =
        timer.schedule_with_delay(chrono::Duration::minutes(options.time_budget), move || {
            let _ = tx.send(());
        });

    match search(&rx, &z3_solver, interpreter.as_mut(), candidate) {
        Some(mut solution) => {
            println!(
                "{}",
                wasmprinter::print_bytes(solution.get_binary()).unwrap()
            );
        }
        None => {
            println!("Failed to superoptimize.");
        }
    }
}

fn search(
    rx: &std::sync::mpsc::Receiver<()>,
    z3_solver: &solver::Z3Solver,
    interpreter: &mut dyn exec::Interpreter,
    candidate: &mut Candidate,
) -> Option<Candidate> {
    let mut whitelist = stoke::whitelist::WHITELIST.to_vec();
    // NOTE(taegyunkim): This is to remove the NOP instruction from the whitelist.
    whitelist.pop();

    for instr in whitelist {
        if rx.try_recv().is_ok() {
            println!("Enumerative search timed out.");
            return None;
        }

        match candidate.try_append(instr) {
            Ok(()) => {
                if candidate.num_values_on_stack == 1 {
                    if interpreter.eval_test_cases(candidate.get_binary()) == 0 {
                        match z3_solver.verify(&candidate.get_func_body()) {
                            solver::VerifyResult::Verified => return Some(candidate.clone()),
                            solver::VerifyResult::CounterExample(values) => {
                                interpreter.add_test_case(values);
                                candidate.drop_last();
                            }
                        }
                    }
                } else {
                    match search(rx, z3_solver, interpreter, candidate) {
                        Some(candidate) => return Some(candidate),
                        None => {
                            candidate.drop_last();
                        }
                    }
                }
            }
            Err(stoke::AppendError::NextIndexOutOfBounds) => {
                return None;
            }
            Err(_) => {
                continue;
            }
        }
    }

    None
}
