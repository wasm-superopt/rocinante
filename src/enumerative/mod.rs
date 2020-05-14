use crate::SuperoptimizerOpts;
use crate::{exec, solver, wasm};
use itertools::Itertools;
use std::collections::HashMap;

pub fn search(
    options: &SuperoptimizerOpts,
    rx: &std::sync::mpsc::Receiver<()>,
    z3_solver: &solver::Z3Solver,
    interpreter: &mut dyn exec::Interpreter,
    spec: &mut wasm::Spec,
) -> Option<wasm::Candidate> {
    let mut seen_skeletons = HashMap::new();
    // min_const, max_const are synthesis bounds for z3
    let (min_const, max_const) = find_min_max(&options.constants);
    let instr_whitelist =
        wasm::Whitelist::new(spec.num_params(), spec.num_locals(), &options.constants);
    let max_length = spec.num_instrs();
    // TODO(taegyunkim): Consider using HashMap instead of vectors for better lookup performance.
    // the ith element in seen_states are obtained from evaluating test cases on ith element in
    // seen candidates.
    let mut seen_candidates: Vec<Vec<parity_wasm::elements::Instruction>> = Vec::new();
    let mut seen_states: Vec<_> = Vec::new();
    // Enumerates programs of length i to max_length
    for i in 1..=max_length {
        // Creates a multi cartesian product of iterators over the whitelisted instructions.
        // For example, if we're given [1, 2, 3], then there are 9 length 2 candidates as following
        // [1, 1], [1, 2], [1, 3].
        // [2, 1], [2, 2], [2, 3],
        // [3, 1], [3, 2], [3, 3],
        // Iterators are cheap to copy and each refer to an element in the vector it was created
        // from.
        seen_skeletons.clear();
        let iter = (0..i)
            .map(|_| instr_whitelist.iter())
            .multi_cartesian_product();
        for candidate in iter {
            if rx.try_recv().is_ok() {
                println!("Enumerative search timed out.");
                return None;
            }

            if let wasm::StackState::Valid = wasm::check_stack_state(&instr_whitelist, &candidate) {
                // Explicitly copy the instruction list to keep track of them.
                let instrs: Vec<parity_wasm::elements::Instruction> =
                    candidate.iter().map(|&item| item.clone()).collect();
                if instrs.len() >= 5
                    && is_new_skeleton(&instrs, &mut seen_skeletons)
                    && contains_const_instrs(&instrs)
                {
                    match z3_solver.synthesize(&instrs, min_const, max_const) {
                        Some(v) => {
                            let synthesized_candidate = wasm::Candidate::from_instrs(v);
                            println!("SAT\nFinished synthesizing:\n{:?}", instrs);
                            // Not necessary, just here to be safe and catch any bugs
                            match z3_solver.verify(synthesized_candidate.instrs()) {
                                solver::VerifyResult::Verified => {
                                    println!("New instructions verified");
                                    return Some(synthesized_candidate);
                                }
                                _ => println!("Not verified. continuing"),
                            }
                        }
                        _ => continue,
                    
                    };
                }
                // Get test outputs returns the output values that are different from the spec, so
                // if this vector is empty, all test cases pass.
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

fn find_min_max(constants: &[i32]) -> (i32, i32) {
    let max_const = constants
        .iter()
        .fold(0, |max, &x| if x > max { x } else { max });
    let min_const = constants
        .iter()
        .fold(0, |min, &x| if x < min { x } else { min });
    (min_const, max_const)
}

// TODO: Find a more efficient way to do this
fn is_new_skeleton(
    instrs: &[parity_wasm::elements::Instruction],
    seen_skeletons: &mut HashMap<String, i32>,
) -> bool {
    let mut s = "".to_string();
    for instr in instrs {
        s.push_str(&match instr {
            parity_wasm::elements::Instruction::I32Const(_) => "i32.const".to_string(),
            parity_wasm::elements::Instruction::GetLocal(_) => "i32.getlocal".to_string(),
            parity_wasm::elements::Instruction::SetLocal(_) => "i32.setlocal".to_string(),
            parity_wasm::elements::Instruction::TeeLocal(_) => "i32.teelocal".to_string(),
            _ => instr.to_string(),
        });
    }
    if !seen_skeletons.contains_key(&s) {
        true
    } else {
        seen_skeletons.insert(s, 0);
        false
    }
}
fn contains_const_instrs(instrs: &[parity_wasm::elements::Instruction]) -> bool {
    for instr in instrs {
        match instr {
            parity_wasm::elements::Instruction::I32Const(_)
            | parity_wasm::elements::Instruction::GetLocal(_)
            | parity_wasm::elements::Instruction::SetLocal(_)
            | parity_wasm::elements::Instruction::TeeLocal(_) => return true,
            _ => false,
        };
    }
    false
}
