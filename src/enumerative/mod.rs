use crate::{exec, solver, stoke, stoke::Candidate};

pub fn search(
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
