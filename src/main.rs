extern crate rocinante;

use clap::{App, Arg, SubCommand};
use rocinante::{exec, stoke};
use std::env;
use std::path::Path;
use std::str;
use std::str::FromStr;

fn parse_module_from_wast(file: &str) -> Vec<Vec<u8>> {
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
    let matches = App::new("Rocinante")
        .author(env!("CARGO_PKG_AUTHORS"))
        .version(env!("CARGO_PKG_VERSION"))
        .about("Superoptimizer for WebAssembly")
        .arg(
            Arg::with_name("FILE")
                .help(".wasm/.wat/.wast file to optimize")
                .required(true)
                .index(1),
        )
        .arg(
            Arg::with_name("algorithm")
                .short("a")
                .help("Superoptimization algorithm to use.")
                .possible_value("Random")
                .possible_value("Stoke")
                .default_value("Stoke"),
        )
        .arg(
            Arg::with_name("interpreter")
                .short("i")
                .help("Which interpreter to use for evaluating test cases.")
                .possible_value("Wasmer")
                .possible_value("Wasmtime")
                .default_value("Wasmer"),
        )
        .arg(
            Arg::with_name("enforce_stack_check_off")
                .short("e")
                .help("Turn off optimization counting values on the stack"),
        )
        .arg(
            Arg::with_name("compute_budget_in_min")
                .short("c")
                .help("The max runtime of one synthesis or optimization step in minutes")
                .default_value("5"),
        )
        .arg(Arg::with_name("beta").short("b").default_value("0.2"))
        .arg(
            Arg::with_name("run_synthesis_only")
                .short("s")
                .help("Run synthesis step only and skip optimization step."),
        )
        .subcommand(
            SubCommand::with_name("print").about("Prints all functions in the given module."),
        )
        .get_matches();

    let input = matches.value_of("FILE").unwrap();
    // Parse the extension of the input file.
    let ext = Path::new(input).extension().unwrap().to_str().unwrap();

    // Read the input file into binary format.
    let binaries: Vec<Vec<u8>> = match ext {
        "wasm" => vec![std::fs::read(input).unwrap()],
        "wat" => vec![wat::parse_file(input).unwrap()],
        "wast" => parse_module_from_wast(input),
        _ => panic!("{}: unrecognized file type", input),
    };

    // TODO(taegyunkim): Parallel processing of different binaries.
    for binary in binaries {
        // Validate raw binary.
        wasmparser::validate(&binary, None /* Uses default parser config */)
            .expect("Failed to validate.");

        // TODO(taegyunkim): Get this from commandline.
        let constants = vec![-2, -1, 0, 1, 2];
        #[cfg(debug_assertions)]
        println!(
            "{}",
            wasmprinter::print_bytes(&binary).expect("Failed to convert to .wat")
        );

        if let Some(_matches) = matches.subcommand_matches("print") {
            continue;
        } else {
            let algorithm =
                stoke::Algorithm::from_str(matches.value_of("algorithm").unwrap()).unwrap();
            let interpreter_kind =
                exec::InterpreterKind::from_str(matches.value_of("interpreter").unwrap()).unwrap();
            let enforce_stack_check = !matches.is_present("enforce_stack_check_off");
            let compute_budget = chrono::Duration::minutes(
                matches
                    .value_of("compute_budget_in_min")
                    .unwrap()
                    .parse::<i64>()
                    .unwrap(),
            );
            let run_synthesis_only = matches.is_present("run_synthesis_only");
            let beta = matches.value_of("beta").unwrap().parse().unwrap();

            let options = stoke::SuperoptimizerOptions::new(
                algorithm,
                interpreter_kind,
                enforce_stack_check,
                compute_budget,
                run_synthesis_only,
                constants,
                beta,
            );
            // TODO(taegyunkim): Propagate the template function.
            let optimizer = stoke::Superoptimizer::new(binary, options);
            optimizer.run();
        }
    }
}
