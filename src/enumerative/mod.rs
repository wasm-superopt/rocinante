use crate::SuperoptimizerOpts;
use crate::{exec, solver, wasm};
use std::collections::BinaryHeap;

pub fn search(
    options: &SuperoptimizerOpts,
    rx: &std::sync::mpsc::Receiver<()>,
    z3_solver: &solver::Z3Solver,
    interpreter: &mut dyn exec::Interpreter,
    spec: &mut wasm::Spec,
) -> Option<wasm::Candidate> {
    let instr_whitelist =
        wasm::Whitelist::new(spec.num_params(), spec.num_locals(), &options.constants);

    // TODO(taegyunkim): Support multiple return values.
    let return_type_len = spec.return_type_len() as i32;

    // A heap to keep track WASM programs in increasing number of instructions.
    let mut candidates: BinaryHeap<wasm::Candidate> = BinaryHeap::new();
    candidates.push(wasm::Candidate::new(spec.num_instrs()));

    let mut seen_states: Vec<_> = Vec::new();
    let mut seen_candidates: Vec<wasm::Candidate> = Vec::new();

    while !candidates.is_empty() {
        if rx.try_recv().is_ok() {
            println!("Enumerative search timed out.");
            return None;
        }

        let mut seen = false;
        let candidate = candidates.pop().unwrap();
        if candidate.num_values_on_stack() == return_type_len {
            let test_outputs =
                interpreter.get_test_outputs(spec.get_binary_with_instrs(candidate.instrs()));

            if test_outputs.is_empty() {
                match z3_solver.verify(&candidate.instrs()) {
                    solver::VerifyResult::Verified => {
                        return Some(candidate);
                    }
                    solver::VerifyResult::CounterExample(values) => {
                        interpreter.add_test_case(values);
                        seen_candidates.push(candidate.clone());
                        seen_states = seen_candidates
                            .iter()
                            .map(|seen_candidate| {
                                interpreter.get_test_outputs(
                                    spec.get_binary_with_instrs(seen_candidate.instrs()),
                                )
                            })
                            .collect();
                    }
                }
            } else {
                match seen_states.iter().position(|s| *s == test_outputs) {
                    Some(idx) => {
                        seen = true;
                        if candidate.instrs().len() < seen_candidates[idx].instrs().len() {
                            seen_candidates[idx] = candidate.clone();
                        }
                    }
                    None => {
                        seen_states.push(test_outputs);
                        seen_candidates.push(candidate.clone());
                    }
                }
            }
        }

        if !seen {
            for instr in instr_whitelist.iter() {
                if let Ok(new_candidate) = candidate.try_append(&instr_whitelist, instr.clone()) {
                    candidates.push(new_candidate);
                }
            }
        }
    }

    None
}
