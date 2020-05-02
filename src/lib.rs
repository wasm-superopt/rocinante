//extern crate bus;
extern crate chrono;
extern crate clap;
extern crate itertools;
#[cfg(test)]
#[macro_use]
extern crate matches;
extern crate num_cpus;
extern crate parity_wasm;
extern crate rand;
extern crate timer;
extern crate wabt;
extern crate wasmer_runtime;
extern crate wasmi;
extern crate wasmparser;
extern crate wasmprinter;
extern crate wast;
extern crate wat;

use std::thread;
//use std::time::Duration;
//use std::sync::mpsc::{sync_channel, TryRecvError};
use crate::exec::InterpreterKind;
use crate::stoke::StokeOpts;
use parity_wasm::elements::{FuncBody, FunctionType, Instruction, Internal, Module};
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use structopt::StructOpt;

pub mod enumerative;
pub mod exec;
pub mod parity_wasm_utils;
pub mod solver;
pub mod stoke;
pub mod wasm;

#[derive(Clone, Debug, StructOpt)]
pub enum Algorithm {
    Stoke(StokeOpts),
    Enumerative,
}

#[derive(Clone, Debug, StructOpt)]
#[structopt(name = "rocinante", about = "WebAssembly Superoptimizer")]
pub struct SuperoptimizerOpts {
    #[structopt(name = "FILE", parse(from_os_str))]
    pub input: PathBuf,

    #[structopt(
        short,
        long,
        help="Which interpreter to use for evaluating test cases.",
        possible_values=&InterpreterKind::variants(),
        default_value="Wasmer")]
    pub interpreter_kind: InterpreterKind,

    #[structopt(
        short,
        long = "no-opti",
        help = "If set, run synthesis step only and skip optimization step, true by default.",
        parse(from_flag = std::ops::Not::not)
    )]
    pub opti: bool,

    #[structopt(
        short,
        long,
        help = "The max runtime of one synthesis or optimization step in minutes.",
        default_value = "5"
    )]
    pub time_budget: i64,

    #[structopt(
        short,
        long,
        help = "A comma separated list of integers for initial set of constants.",
        default_value = "-2,-1,0,1,2",
        require_delimiter(true)
    )]
    pub constants: Vec<i32>,

    #[structopt(subcommand)]
    pub algorithm: Algorithm,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum Mode {
    Synthesis,
    Optimization,
}

pub struct Superoptimizer {
    spec: Vec<u8>,
    options: SuperoptimizerOpts,
}

impl Superoptimizer {
    pub fn new(spec: Vec<u8>, options: SuperoptimizerOpts) -> Self {
        Superoptimizer { spec, options }
    }

    pub fn run(&self) {
        let module = Module::from_bytes(&self.spec).unwrap();

        // TODO(taegyunkim): Use num_cpus crate to appropriately set the number of workers.
        let num_workers = num_cpus::get();
        let candidates = Arc::new(Mutex::new(Vec::with_capacity(num_workers)));

        let export_section = module
            .export_section()
            .expect("Module doesn't have export section.");

        for export_entry in export_section.entries() {
            if let Internal::Function(_idx) = export_entry.internal() {
                let mut threads = Vec::new();
                // TODO(taegyunkim): Parallel processing.
                for _ in 0..num_workers {
                    let func_name = export_entry.field().to_string();
                    let tmp_options = self.options.clone();
                    let tmp_spec = self.spec.clone();
                    let module2 = module.clone();
                    // let res_sender_i = res_sender.clone();
                    threads.push(thread::spawn({
                        let candidates_clone = Arc::clone(&candidates);
                        move || {
                            let (func_type, func_body) =
                                parity_wasm_utils::func_by_name(&module2, &func_name);
                            if let Some(mut candidate) = invoke_search(
                                &tmp_spec,
                                &func_name,
                                func_type,
                                func_body,
                                &tmp_options,
                                Mode::Synthesis,
                            ) {
                                candidate.strip_nops();
                                let mut locked_candidates = candidates_clone.lock().unwrap();
                                locked_candidates.push(candidate);

                                if tmp_options.opti {
                                    if let Some(mut candidate) = invoke_search(
                                        &tmp_spec,
                                        &func_name,
                                        func_type,
                                        func_body,
                                        &tmp_options,
                                        Mode::Optimization,
                                    ) {
                                        candidate.strip_nops();
                                        let mut locked_candidates =
                                            candidates_clone.lock().unwrap();
                                        locked_candidates.push(candidate);
                                    }
                                }
                            }

                            // Signal here
                        }
                    }));
                }
                for t in threads {
                    t.join().unwrap();
                }
            }
        }
        let locked_clone = Arc::clone(&candidates);
        rank(&locked_clone.lock().unwrap());
    }
}
fn invoke_search(
    spec: &[u8],
    func_name: &str,
    func_type: &FunctionType,
    func_body: &FuncBody,
    options: &SuperoptimizerOpts,
    mode: Mode,
) -> Option<wasm::Candidate> {
    // NOTE(taegyunkim): Interpreter is not thread safe.
    let mut interpreter = exec::get_interpreter(options.interpreter_kind, spec, func_name);

    let mut spec = wasm::Spec::new(func_type, func_body);

    let cfg = z3::Config::new();
    let ctx = z3::Context::new(&cfg);
    let z3_solver = solver::Z3Solver::new(&ctx, func_type, func_body);

    // Timer to terminate the search after given computing budget.
    let timer = timer::Timer::new();
    let (tx, rx) = std::sync::mpsc::channel();
    // It's necessary to name this variable to trigger the callback.
    let _guard =
        timer.schedule_with_delay(chrono::Duration::minutes(options.time_budget), move || {
            let _ = tx.send(());
        });

    match &options.algorithm {
        Algorithm::Stoke(stoke_options) => stoke::search(
            options,
            stoke_options,
            mode,
            &rx,
            &z3_solver,
            interpreter.as_mut(),
            &mut spec,
        ),
        Algorithm::Enumerative => {
            enumerative::search(options, &rx, &z3_solver, interpreter.as_mut(), &mut spec)
        }
    }
}
pub fn rank(candidates: &[wasm::Candidate]) {
    println!("Found {} programs", candidates.len());

    let best = candidates
        .iter()
        .min_by(|a, b| perf(a.instrs()).cmp(&perf(b.instrs())))
        .unwrap();

    println!("{:?}", best.instrs());
}

pub fn perf(instrs: &[Instruction]) -> u32 {
    let mut cnt = 0;
    for instr in instrs {
        if *instr != Instruction::Nop {
            cnt += 1;
        }
    }
    cnt
}
