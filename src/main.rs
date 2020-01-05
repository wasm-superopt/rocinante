extern crate clap;
extern crate wabt;
extern crate wat;

use clap::{App, Arg};
use std::env;
use std::fs::File;
use std::io;
use std::io::prelude::*;
use std::path::Path;
use std::str;
use wabt::{Module, ReadBinaryOptions};

fn read_wasm(file: &str) -> io::Result<Vec<u8>> {
    let mut data = Vec::new();
    let mut f = File::open(file)?;
    f.read_to_end(&mut data)?;
    Ok(data)
}

fn main() {
    let matches = App::new("Rocinante")
        .version(clap::crate_version!())
        .author("Taegyun Kim <k.taegyun@gmail.com>")
        .about("Superoptimizer for WebAssembly")
        .arg(
            Arg::with_name("FILE")
                .help(".wasm/.wat/.wast file to optimize")
                .required(true)
                .index(1),
        )
        .get_matches();

    let _script_ext = "wast";

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
}
