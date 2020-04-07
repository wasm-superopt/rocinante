use crate::candidate as new_candidate;
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

    let num_locals = candidate.spec_func_type().params().len() + candidate.spec_local_types.len();

    for idx in 0..num_locals as u32 {
        whitelist.push(Instruction::GetLocal(idx));
        whitelist.push(Instruction::SetLocal(idx));
        whitelist.push(Instruction::TeeLocal(idx));
    }

    for constant in &candidate.constants {
        whitelist.push(Instruction::I32Const(*constant));
    }

    let mut rng = rand::thread_rng();

    let mut candidates: BinaryHeap<new_candidate::Candidate> = BinaryHeap::new();
    candidates.push(new_candidate::Candidate::new(candidate.instrs().len()));

    while !candidates.is_empty() {
        if rx.try_recv().is_ok() {
            println!("Enumerative search timed out.");
            return None;
        }

        let program = candidates.pop().unwrap();
        if program.num_values_on_stack() == program.return_type_len() {
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
            match program.try_append(instr.clone()) {
                Ok(new_program) => {
                    candidates.push(new_program);
                }
                Err(_) => {}
            }
        }
        whitelist.shuffle(&mut rng);
    }

    None
}
