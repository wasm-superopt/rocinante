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

    let mut candidates: BinaryHeap<wasm::Candidate> = BinaryHeap::new();
    candidates.push(wasm::Candidate::new(spec.num_instrs()));

    while !candidates.is_empty() {
        if rx.try_recv().is_ok() {
            println!("Enumerative search timed out.");
            return None;
        }

        let candidate = candidates.pop().unwrap();
        if candidate.num_values_on_stack() == return_type_len
            && interpreter.eval_test_cases(spec.get_binary_with_instrs(candidate.instrs())) == 0
        {
            match z3_solver.verify(&candidate.instrs()) {
                solver::VerifyResult::Verified => {
                    return Some(candidate);
                }
                solver::VerifyResult::CounterExample(values) => {
                    interpreter.add_test_case(values);
                }
            }
        }

        for instr in instr_whitelist.iter() {
            if let Ok(new_candidate) = candidate.try_append(&instr_whitelist, instr.clone()) {
                candidates.push(new_candidate);
            }
        }
    }

    None
}
