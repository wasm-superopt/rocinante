use crate::wasm::Candidate;
use crate::wasm::Spec;
use crate::wasm::Whitelist;
use crate::SuperoptimizerOpts;
use crate::{exec, solver};
use std::collections::BinaryHeap;

pub fn search(
    options: &SuperoptimizerOpts,
    rx: &std::sync::mpsc::Receiver<()>,
    z3_solver: &solver::Z3Solver,
    interpreter: &mut dyn exec::Interpreter,
    spec: &mut Spec,
) -> Option<Spec> {
    let instr_whitelist = Whitelist::new(spec.num_params(), spec.num_locals(), &options.constants);

    // TODO(taegyunkim): Support multiple return values.
    let return_type_len = 1;

    let mut candidates: BinaryHeap<Candidate> = BinaryHeap::new();
    candidates.push(Candidate::new(spec.instrs().len()));

    while !candidates.is_empty() {
        if rx.try_recv().is_ok() {
            println!("Enumerative search timed out.");
            return None;
        }

        let program = candidates.pop().unwrap();
        if program.num_values_on_stack() == return_type_len {
            spec.instrs_mut().clone_from_slice(program.instrs());
            if interpreter.eval_test_cases(spec.get_binary()) == 0 {
                match z3_solver.verify(&spec.get_func_body()) {
                    solver::VerifyResult::Verified => {
                        return Some(spec.clone());
                    }
                    solver::VerifyResult::CounterExample(values) => {
                        interpreter.add_test_case(values);
                    }
                }
            }
        }

        for instr in instr_whitelist.iter() {
            if let Ok(new_program) = program.try_append(&instr_whitelist, instr.clone()) {
                candidates.push(new_program);
            }
        }
    }

    None
}
