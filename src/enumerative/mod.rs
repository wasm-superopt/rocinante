use crate::exec;
use crate::solver;
use crate::stoke::Candidate;
use crate::SuperoptimizerOpts;

pub fn run(spec: Vec<u8>, candidate: &mut Candidate, options: SuperoptimizerOpts, func_name: &str) {
    let _interpreter = exec::get_interpreter(options.interpreter_kind, &spec, func_name);

    let spec_func_type = candidate.spec_func_type();
    let spec_func_body = candidate.get_spec_func_body();

    let cfg = z3::Config::new();
    let ctx = z3::Context::new(&cfg);
    let z3solver = solver::Z3Solver::new(&ctx, spec_func_type, spec_func_body);

    // Set up timer and channels to send time outs.
    let timer = timer::Timer::new();
    let (tx, rx) = std::sync::mpsc::channel();
    let _guard =
        timer.schedule_with_delay(chrono::Duration::minutes(options.time_budget), move || {
            let _ = tx.send(());
        });

    loop {
        match search(candidate) {
            Some(mut candidate) => match z3solver.verify(&candidate.get_func_body()) {
                solver::VerifyResult::Verified => {
                    println!(
                        "{}",
                        wasmprinter::print_bytes(candidate.get_binary()).unwrap()
                    );
                    break;
                }
                solver::VerifyResult::CounterExample(_values) => {}
            },
            None => {
                println!("Failed to symthesize");
                break;
            }
        }
        if rx.try_recv().is_ok() {
            println!("enumerative search timed out");
            break;
        }
    }
}

fn search(_candidate: &mut Candidate) -> Option<Candidate> {
    None
}
