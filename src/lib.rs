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
#[structopt(name = "options", about = "Superoptimizer options.")]
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
