#[macro_use]
extern crate itertools;
extern crate clap;
extern crate parity_wasm;
extern crate wabt;
extern crate wasmparser;
extern crate wat;

use clap::{App, Arg};
use parity_wasm::elements::{Module, Type};
use std::env;
use std::fs::File;
use std::io;
use std::io::prelude::*;
use std::path::Path;
use std::str;

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

    // Deserialize into an IR using parity-wasm.
    let module = Module::from_bytes(&binary).expect("Failed to deserialize.");

    let function_section = module
        .function_section()
        .expect("No function section in the module.");

    let code_section = module
        .code_section()
        .expect("No code section in the module.");

    let type_section = module
        .type_section()
        .expect("No type section in the module.");

    // We assume that the number of function signatures and function bodies are
    // the same in the module and the ordering is also the same.
    assert!(function_section.entries().len() == code_section.bodies().len());

    for (func, body) in iproduct!(function_section.entries(), code_section.bodies()) {
        let type_idx = func.type_ref();
        let typ = &type_section.types()[type_idx as usize];
        // Type section only contains function types, so coerce it to a
        // FunctionType.
        let Type::Function(func_type) = typ;
        for (i, param_type) in func_type.params().iter().enumerate() {
            println!("param {}, type: {}", i, param_type);
        }

        if let Some(return_type) = func_type.return_type() {
            println!("return type: {}", return_type);
        }

        for local in body.locals() {
            println!("local {}, type: {}", local.count(), local.value_type());
        }
        for instr in body.code().elements() {
            println!("{}", instr);
        }
    }
}
