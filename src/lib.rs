extern crate chrono;
extern crate clap;
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

use crate::exec::InterpreterKind;
use crate::stoke::StokeOpts;
use parity_wasm::elements::{Instruction, Internal, Module};
use std::path::PathBuf;
use structopt::StructOpt;

pub mod exec;
pub mod parity_wasm_utils;
pub mod solver;
pub mod stoke;

#[derive(Clone, Debug, StructOpt)]
pub enum Algorithm {
    Stoke(StokeOpts),
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
    stoke_options: StokeOpts,
    options: SuperoptimizerOpts,
}

impl Superoptimizer {
    pub fn new(spec: Vec<u8>, options: SuperoptimizerOpts) -> Self {
        let Algorithm::Stoke(stoke_options) = options.algorithm.clone();
        Superoptimizer {
            spec,
            stoke_options,
            options,
        }
    }

    pub fn run(&self) {
        let module = Module::from_bytes(&self.spec).unwrap();

        // TODO(taegyunkim): Use num_cpus crate to appropriately set the number of workers.
        let num_workers = 1;
        let mut candidates: Vec<stoke::Candidate> = Vec::with_capacity(num_workers);

        let export_section = module
            .export_section()
            .expect("Module doesn't have export section.");

        for export_entry in export_section.entries() {
            if let Internal::Function(_idx) = export_entry.internal() {
                let func_name = export_entry.field();

                let (func_type, func_body) = parity_wasm_utils::func_by_name(&module, func_name);

                // TODO(taegyunkim): Parallel processing.
                for _ in 0..num_workers {
                    // NOTE(taegyunkim): Interpreter is not thread safe.
                    let mut interpreter =
                        exec::get_interpreter(self.options.interpreter_kind, &self.spec, func_name);

                    let mut candidate =
                        stoke::Candidate::new(func_type, func_body, self.options.constants.clone());

                    if stoke::do_run(
                        &self.options,
                        &self.stoke_options,
                        Mode::Synthesis,
                        interpreter.as_mut(),
                        &mut candidate,
                    ) {
                        candidate.strip_nops();
                        candidates.push(candidate.clone());

                        if !self.options.opti {
                            continue;
                        }

                        if stoke::do_run(
                            &self.options,
                            &self.stoke_options,
                            Mode::Optimization,
                            interpreter.as_mut(),
                            &mut candidate,
                        ) {
                            candidate.strip_nops();
                            candidates.push(candidate);
                        }
                    }
                }
            }
        }

        rank(&candidates);
    }
}

pub fn rank(candidates: &[stoke::Candidate]) {
    println!("Found {} programs", candidates.len());

    let best = candidates
        .iter()
        .min_by(|a, b| perf(a.instrs()).cmp(&perf(b.instrs())))
        .unwrap();

    println!(
        "{}",
        wasmprinter::print_bytes(best.to_module().to_bytes().unwrap()).unwrap()
    );
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
