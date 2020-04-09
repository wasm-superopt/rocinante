use crate::wasm::candidate as new_candidate;
use crate::{exec, solver, stoke, stoke::Candidate};
use parity_wasm::elements::Instruction;
use rand::seq::SliceRandom;
use std::collections::BinaryHeap;

pub fn search(
    rx: &std::sync::mpsc::Receiver<()>,
    z3_solver: &solver::Z3Solver,
    interpreter: &mut dyn exec::Interpreter,
    candidate: &mut Candidate,
) -> Option<Candidate> {
    let mut whitelist = stoke::whitelist::I32BINOP.to_vec();

    whitelist.append(&mut stoke::whitelist::I32UNOP.to_vec());
    whitelist.append(&mut stoke::whitelist::I32RELOP.to_vec());

    let num_local_indices = candidate.num_locals() + candidate.num_params();

    for idx in 0..num_local_indices as u32 {
        whitelist.push(Instruction::GetLocal(idx));
        whitelist.push(Instruction::SetLocal(idx));
        whitelist.push(Instruction::TeeLocal(idx));
    }

    for constant in &candidate.constants {
        whitelist.push(Instruction::I32Const(*constant));
    }

    let mut rng = rand::thread_rng();

    // TODO(taegyunkim): Support multiple return values.
    let return_type_len = 1;

    let mut candidates: BinaryHeap<new_candidate::Candidate> = BinaryHeap::new();
    candidates.push(new_candidate::Candidate::new(candidate.instrs().len()));

    while !candidates.is_empty() {
        if rx.try_recv().is_ok() {
            println!("Enumerative search timed out.");
            return None;
        }

        let program = candidates.pop().unwrap();
        if program.num_values_on_stack() == return_type_len {
            candidate.instrs_mut().clone_from_slice(program.instrs());
            if interpreter.eval_test_cases(candidate.get_binary()) == 0 {
                match z3_solver.verify(&candidate.get_func_body()) {
                    solver::VerifyResult::Verified => {
                        return Some(candidate.clone());
                    }
                    solver::VerifyResult::CounterExample(values) => {
                        interpreter.add_test_case(values);
                    }
                }
            }
        }

        for instr in &whitelist {
            if let Ok(new_program) = program.try_append(instr.clone()) {
                candidates.push(new_program);
            }
        }
        whitelist.shuffle(&mut rng);
    }

    None
}
