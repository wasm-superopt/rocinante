extern crate env_logger;
extern crate log;
extern crate z3;
use std::convert::TryInto;
use z3::ast::Ast;
use z3::*;

fn main() {
    println!("Hello, world; trying z3 example!");
    let _ = env_logger::try_init();
    let cfg = Config::new();
    let ctx = Context::new(&cfg);
    let x = ast::Int::new_const(&ctx, "x");
    let y = ast::Int::new_const(&ctx, "y");

    let solver = Solver::new(&ctx);
    solver.assert(&x.gt(&y));
    assert_eq!(solver.check(), SatResult::Sat);
}
