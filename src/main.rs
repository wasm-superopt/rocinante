extern crate rocinante;

use rocinante::stoke;
use std::path::Path;
use structopt::StructOpt;

fn parse_module_from_wast(file: impl AsRef<Path>) -> Vec<Vec<u8>> {
    let contents = std::fs::read_to_string(file).unwrap();
    let buf = wast::parser::ParseBuffer::new(&contents).unwrap();
    let wast = wast::parser::parse::<wast::Wast>(&buf).unwrap();

    let mut modules: Vec<Vec<u8>> = Vec::new();
    for directive in wast.directives {
        // NOTE(taegyunkim): Other directives can have modules in them, but
        // we're ingorning them for now.
        if let wast::WastDirective::Module(mut module) = directive {
            modules.push(module.encode().unwrap());
        }
    }
    modules
}

fn main() {
    let options = rocinante::SuperoptimizerOpts::from_args();

    // Parse the extension of the input file.
    let ext = Path::new(&options.input)
        .extension()
        .unwrap()
        .to_str()
        .unwrap();

    // Read the input file into binary format.
    let binaries: Vec<Vec<u8>> = match ext {
        "wasm" => vec![std::fs::read(&options.input).unwrap()],
        "wat" => vec![wat::parse_file(&options.input).unwrap()],
        "wast" => parse_module_from_wast(&options.input),
        _ => panic!(
            "{}: unrecognized file type",
            options.input.to_str().unwrap()
        ),
    };

    // TODO(taegyunkim): Parallel processing of different binaries.
    for binary in binaries {
        // Validate raw binary.
        wasmparser::validate(&binary, None /* Uses default parser config */)
            .expect("Failed to validate.");

        println!(
            "{}",
            wasmprinter::print_bytes(&binary).expect("Failed to convert to .wat")
        );
        // TODO(taegyunkim): Propagate the template function.
        let optimizer = stoke::Superoptimizer::new(binary, options.clone());
        optimizer.run();
    }
}
