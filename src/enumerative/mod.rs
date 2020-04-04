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

        println!(
            "{}",
            wasmprinter::print_bytes(candidate.to_module().to_bytes().unwrap()).unwrap()
        );
        match candidate.try_append(instr) {
            Ok(()) => {
                println!("num values on stack: {}", candidate.num_values_on_stack);
                if candidate.num_values_on_stack == 1 {
                    if interpreter.eval_test_cases(candidate.get_binary()) == 0 {
                        println!("Passed all test cases.");
                        match z3_solver.verify(&candidate.get_func_body()) {
                            solver::VerifyResult::Verified => {
                                println!("Verified");
                                return Some(candidate.clone());
                            }
                            solver::VerifyResult::CounterExample(values) => {
                                println!("Added counter example");
                                interpreter.add_test_case(values);
                            }
                        }
                    } else {
                        println!("didn't pass all the test cases.");

                        println!(
                            "{}",
                            wasmprinter::print_bytes(candidate.to_module().to_bytes().unwrap())
                                .unwrap()
                        );
                    }
                }
                match search(rx, z3_solver, interpreter, candidate) {
                    Some(candidate) => return Some(candidate),
                    None => {
                        candidate.drop_last();
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
