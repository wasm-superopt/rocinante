use crate::SuperoptimizerOpts;
use crate::{exec, solver, wasm};
use itertools::Itertools;

pub fn search(
    options: &SuperoptimizerOpts,
    rx: &std::sync::mpsc::Receiver<()>,
    z3_solver: &solver::Z3Solver,
    interpreter: &mut dyn exec::Interpreter,
    spec: &mut wasm::Spec,
) -> Option<wasm::Candidate> {
    let instr_whitelist =
        wasm::Whitelist::new(spec.num_params(), spec.num_locals(), &options.constants);

    let max_length = spec.num_instrs();

    // TODO(taegyunkim): Consider using HashMap instead of vectors for better lookup performance.
    let mut seen_states: Vec<_> = Vec::new();
    let mut seen_candidates: Vec<Vec<parity_wasm::elements::Instruction>> = Vec::new();

    for i in 1..=max_length {
        let iter = (0..i)
            .map(|_| instr_whitelist.iter())
            .multi_cartesian_product();

        for candidate in iter {
            if rx.try_recv().is_ok() {
                println!("Enumerative search timed out.");
                return None;
            }

            if let wasm::StackState::Valid = wasm::check_stack_state(&instr_whitelist, &candidate) {
                let instrs: Vec<parity_wasm::elements::Instruction> =
                    candidate.iter().map(|&item| item.clone()).collect();
                let test_outputs =
                    interpreter.get_test_outputs(spec.get_binary_with_instrs(&instrs));
                if test_outputs.is_empty() {
                    match z3_solver.verify(&instrs) {
                        solver::VerifyResult::Verified => {
                            return Some(wasm::Candidate::from_instrs(instrs));
                        }
                        solver::VerifyResult::CounterExample(values) => {
                            interpreter.add_test_case(values);
                            seen_candidates.push(instrs);
                            seen_states = seen_candidates
                                .iter()
                                .map(|seen_candidate| {
                                    interpreter.get_test_outputs(
                                        spec.get_binary_with_instrs(seen_candidate),
                                    )
                                })
                                .collect();
                        }
                    }
                } else {
                    match seen_states.iter().position(|s| *s == test_outputs) {
                        Some(idx) => {
                            if instrs.len() < seen_candidates[idx].len() {
                                seen_candidates[idx] = instrs;
                            }
                        }
                        None => {
                            seen_states.push(test_outputs);
                            seen_candidates.push(instrs);
                        }
                    }
                }
            }
        }
    }

    None
}
