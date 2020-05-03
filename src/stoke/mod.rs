use crate::wasm::{Candidate, Spec, StackState};
use crate::{exec, perf, solver, wasm, Mode, SuperoptimizerOpts};
use clap::arg_enum;
use rand::distributions::{Bernoulli, Distribution};
use structopt::StructOpt;

use self::transform::*;
pub mod transform;

arg_enum! {
    #[derive(Clone, Debug)]
    pub enum Sampler {
        Random,
        MCMC,
    }
}

#[derive(Clone, Debug, StructOpt)]
#[structopt(name = "stoke_opts", about = "Stochastic search specific options.")]
pub struct StokeOpts {
    #[structopt(
        short,
        long,
        help="The sampler algorithm to use",
        possible_values=&Sampler::variants(),
        default_value="MCMC")]
    pub sampler: Sampler,

    #[structopt(short, long="no-enforce-stack-check", parse(from_flag = std::ops::Not::not))]
    pub enforce_stack_check: bool,

    #[structopt(short, long, default_value = "0.2")]
    pub beta: f64,
}

fn eval_candidate(
    stoke_options: &StokeOpts,
    mode: Mode,
    instr_whitelist: &wasm::Whitelist,
    interpreter: &dyn exec::Interpreter,
    spec: &mut Spec,
    candidate: &Candidate,
) -> u32 {
    let mut cost = if stoke_options.enforce_stack_check {
        match candidate.is_stack_valid(instr_whitelist) {
            StackState::Valid => {
                let binary = spec.get_binary_with_instrs(candidate.instrs());
                interpreter.eval_test_cases(&binary)
            }
            StackState::Invalid(cnt) => {
                // If the program is invalid we penalize it the stack value count difference.
                interpreter.score_invalid()
                    + (i32::abs(interpreter.return_type_len() as i32 - cnt) as u32 + 1)
            }
        }
    } else {
        let binary = spec.get_binary_with_instrs(candidate.instrs());
        interpreter.eval_test_cases(&binary)
    };

    if mode == Mode::Optimization {
        cost += perf(candidate.instrs());
    }

    cost
}

pub fn search(
    options: &SuperoptimizerOpts,
    stoke_options: &StokeOpts,
    mode: Mode,
    rx: &std::sync::mpsc::Receiver<()>,
    z3_solver: &solver::Z3Solver,
    interpreter: &mut dyn exec::Interpreter,
    spec: &mut Spec,
) -> Option<Candidate> {
    let mut rng = rand::thread_rng();

    let instr_whitelist =
        wasm::Whitelist::new(spec.num_params(), spec.num_locals(), &options.constants);

    let mut candidate = Candidate::new(spec.num_instrs());

    let transform = Transform::new(spec.param_types().to_vec(), spec.local_types().to_vec());

    let mut curr_cost = eval_candidate(
        stoke_options,
        mode,
        &instr_whitelist,
        interpreter,
        spec,
        &candidate,
    );
    let initial_cost = curr_cost;

    loop {
        if (mode == Mode::Optimization && curr_cost < initial_cost)
            || (mode == Mode::Synthesis && curr_cost == 0)
        {
            match z3_solver.verify(&candidate.instrs()) {
                solver::VerifyResult::Verified => {
                    println!("ACCEPTED! Now cancelling all other threads ... ");
                    return Some(candidate);
                }
                solver::VerifyResult::CounterExample(values) => {
                    interpreter.add_test_case(values);
                    curr_cost = interpreter.return_bit_width();
                }
            }
        }

        if rx.try_recv().is_ok() {
            println!("Stochastic search {:?} timed out", mode);
            break;
        }

        let transform_info = transform.operate(&mut rng, &instr_whitelist, &mut candidate);
        let new_cost = eval_candidate(
            stoke_options,
            mode,
            &instr_whitelist,
            interpreter,
            spec,
            &candidate,
        );

        #[cfg(debug_assertions)]
        println!("curr_cost: {}, new_cost: {}", curr_cost, new_cost);
        match stoke_options.sampler {
            Sampler::Random => {
                // Always accept transform.
                curr_cost = new_cost;
            }
            Sampler::MCMC => {
                if new_cost < curr_cost {
                    // Accept this transform.
                    curr_cost = new_cost;
                } else {
                    // Following computes min(1, exp(-0.4 * new_cost/ curr_cost))
                    // TODO(taegyunkim): Use parameter \beta instead of -0.4
                    let p: f64 = (1.0 as f64)
                        .min((-stoke_options.beta * (new_cost as f64) / (curr_cost as f64)).exp());
                    let d = Bernoulli::new(p).unwrap();
                    #[cfg(debug_assertions)]
                    println!("p: {}", p);
                    let accept = d.sample(&mut rng);
                    if !accept {
                        #[cfg(debug_assertions)]
                        println!("undoing...");
                        transform.undo(&transform_info, &mut candidate);
                    } else {
                        #[cfg(debug_assertions)]
                        println!("accepted...");
                        curr_cost = new_cost;
                    }
                }
            }
        }
    }

    None
}
