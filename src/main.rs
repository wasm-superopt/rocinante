extern crate clap;
#[cfg(test)]
#[macro_use]
extern crate matches;
extern crate parity_wasm;
extern crate rand;
extern crate wabt;
extern crate wasmi;
extern crate wasmparser;
extern crate wat;

use clap::{App, Arg, SubCommand};
use parity_wasm::elements::Module;
use std::env;
use std::fs::File;
use std::io;
use std::io::prelude::*;
use std::path::Path;
use std::str;

pub mod debug;
pub mod exec;
mod parity_wasm_utils;
pub mod solver;
pub mod stoke;
mod wasmi_utils;

fn read_wasm(file: &str) -> io::Result<Vec<u8>> {
    let mut data = Vec::new();
    let mut f = File::open(file)?;
    f.read_to_end(&mut data)?;
    Ok(data)
}

fn main() {
    let matches = App::new("Rocinante")
        .author(env!("CARGO_PKG_AUTHORS"))
        .version(env!("CARGO_PKG_VERSION"))
        .about("Superoptimizer for WebAssembly")
        .arg(
            Arg::with_name("FILE")
                .help(".wasm/.wat file to optimize")
                .required(true)
                .index(1),
        )
        .subcommand(
            SubCommand::with_name("print").about("Prints all functions in the given module."),
        )
        .get_matches();

    let input = matches.value_of("FILE").unwrap();
    // Parse the extension of the input file.
    let ext = Path::new(input).extension().unwrap().to_str().unwrap();

    // Read the input file into binary format.
    // TODO: Consider supporting wast. wast is a superset of wat that is
    // intended for writing test  scripts, and can contain assertions and other
    // commands.
    let binary: Vec<u8> = match ext {
        "wasm" => read_wasm(input).unwrap(),
        "wat" => wat::parse_file(input).unwrap(),
        "wast" | _ => panic!("{}: unrecognized file type", input),
    };

    // Validate raw binary.
    wasmparser::validate(&binary, None /* Uses default parser config */)
        .expect("Failed to validate.");

    // Deserialize into an IR using parity-wasm.
    let module = Module::from_bytes(&binary).expect("Failed to deserialize.");

    if let Some(_matches) = matches.subcommand_matches("print") {
        debug::print_functions(&module);
    } else {
        let optimizer = stoke::Optimizer::new(module);
        optimizer.run();
    }
}
