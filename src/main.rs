extern crate clap;
extern crate wabt;
extern crate wasmparser;
extern crate wat;

use clap::{App, Arg};
use std::env;
use std::fs::File;
use std::io;
use std::io::prelude::*;
use std::path::Path;
use std::str;
use wabt::{Module, ReadBinaryOptions};
use wasmparser::Parser;
use wasmparser::ParserState;
use wasmparser::WasmDecoder;

fn read_wasm(file: &str) -> io::Result<Vec<u8>> {
    let mut data = Vec::new();
    let mut f = File::open(file)?;
    f.read_to_end(&mut data)?;
    Ok(data)
}

fn main() {
    let matches = App::new("Rocinante")
        .version(clap::crate_version!())
        .about("Superoptimizer for WebAssembly")
        .arg(
            Arg::with_name("FILE")
                .help(".wasm/.wat/.wast file to optimize")
                .required(true)
                .index(1),
        )
        .get_matches();

    let input = matches.value_of("FILE").unwrap();
    let ext = Path::new(input).extension().unwrap().to_str().unwrap();

    // TODO: Consider supporting wast. wast is a superset of wat that is
    // intended for writing test  scripts, and can contain assertions and other
    // commands.
    let binary: Vec<u8> = match ext {
        "wasm" => read_wasm(input).unwrap(),
        "wat" => wat::parse_file(input).unwrap(),
        "wast" | _ => panic!("{}: unrecognized file type", input),
    };

    let module = Module::read_binary(&binary, &ReadBinaryOptions::default()).unwrap();
    module.validate().unwrap();

    let mut parser = Parser::new(&binary);

    loop {
        print!("0x{:08x}\t", parser.current_position());
        let state = parser.read();
        match *state {
            ParserState::ExportSectionEntry {
                field,
                ref kind,
                index,
            } => {
                println!(
                    "ExportSectionEntry {{ field: \"{}\", kind: {:?}, index: {} }}",
                    field, kind, index
                );
            }
            ParserState::ImportSectionEntry {
                module,
                field,
                ref ty,
            } => {
                println!(
                    "ImportSectionEntry {{ module: \"{}\", field: \"{}\", ty: {:?} }}",
                    module, field, ty
                );
            }
            ParserState::EndWasm => break,
            ParserState::Error(err) => panic!("Error: {:?}", err),
            _ => println!("{:?}", state),
        }
    }
}
