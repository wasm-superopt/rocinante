extern crate clap;
extern crate itertools;
extern crate parity_wasm;
extern crate wabt;
extern crate wasmparser;
extern crate wat;

use clap::{App, Arg};
use parity_wasm::elements::{ExportSection, Internal, Module};
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

fn get_func_name(export_section: &ExportSection, func_idx: u32) -> Option<&str> {
    let export_entries = export_section.entries();

    for entry in export_entries {
        if let Internal::Function(idx) = entry.internal() {
            if *idx == func_idx {
                return Some(entry.field());
            }
        }
    }

    None
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

    // Validate raw binary.
    wasmparser::validate(&binary, None /* Uses default parser config */)
        .expect("Failed to validate.");

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

    let export_section = module
        .export_section()
        .expect("No export section in the module.");

    // We assume that the number of function signatures and function bodies are
    // the same in the module and the ordering is also the same.
    assert!(function_section.entries().len() == code_section.bodies().len());

    let num_func = function_section.entries().len();
    for i in 0..num_func {
        let func_sig = function_section.entries()[i];
        let typ_idx = func_sig.type_ref();
        let typ = &type_section.types()[typ_idx as usize];

        let name_opt = get_func_name(export_section, i as u32);

        if let Some(func_name) = name_opt {
            println!("{}", func_name);
        }

        println!("{:?}", typ);

        let func_body = &code_section.bodies()[i];
        println!("{:?}", func_body.locals());

        for instr in func_body.code().elements() {
            println!("{}", instr);
        }

        println!();
    }
}
