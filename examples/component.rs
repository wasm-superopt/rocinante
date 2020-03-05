extern crate env_logger;
extern crate log;
extern crate z3;

use std::collections::HashMap;
use z3::{
    ast::{Ast, Bool, Datatype, Int, BV},
    Config, Context, DatatypeBuilder, DatatypeSort, SatResult, Solver,
};

// Function that converts an Instruction datatype into z3::ast::Int which represents the number
// of values the instruction pops off from the stack.
fn pop_cnts<'a>(ctx: &'a Context, instr_type: &DatatypeSort<'a>, instr: &Datatype<'a>) -> Int<'a> {
    instr_type.variants[0]
        .tester
        .apply(&[&instr.clone().into()])
        .as_bool()
        .unwrap()
        .ite(
            &(Int::from_u64(&ctx, 0)),
            &instr_type.variants[1]
                .tester
                .apply(&[&instr.clone().into()])
                .as_bool()
                .unwrap()
                .ite(&(Int::from_u64(&ctx, 2)), &(Int::from_u64(&ctx, 0))),
        )
}

#[allow(dead_code)]
fn pop_cnts_map<'a>(
    ctx: &'a Context,
    instr_type: &DatatypeSort<'a>,
    instr: &Datatype<'a>,
) -> Int<'a> {
    // Initialize a map from datatype values to pop counts.
    let mut instrs_to_cnts = HashMap::new();

    instrs_to_cnts.insert(
        instr_type.variants[0]
            .constructor
            .apply(&[])
            .as_datatype()
            .unwrap(),
        0,
    );
    instrs_to_cnts.insert(
        instr_type.variants[1]
            .constructor
            .apply(&[])
            .as_datatype()
            .unwrap(),
        2,
    );
    instrs_to_cnts.insert(
        instr_type.variants[2]
            .constructor
            .apply(&[])
            .as_datatype()
            .unwrap(),
        0,
    );

    match instrs_to_cnts.get(instr) {
        Some(i) => Int::from_u64(&ctx, *i),
        None => panic!("pop counts map doesn't contain key"),
    }
}

// Function that converts an Instruction datatype into z3::ast::Int which represents the number
// of values the instruction pushes to the stack.
fn push_cnts<'a>(ctx: &'a Context, instr_type: &DatatypeSort<'a>, instr: &Datatype<'a>) -> Int<'a> {
    instr_type.variants[0]
        .tester
        .apply(&[&instr.clone().into()])
        .as_bool()
        .unwrap()
        .ite(
            &(Int::from_u64(&ctx, 1)),
            &instr_type.variants[1]
                .tester
                .apply(&[&instr.clone().into()])
                .as_bool()
                .unwrap()
                .ite(&(Int::from_u64(&ctx, 1)), &(Int::from_u64(&ctx, 0))),
        )
}

/// A helper function for is_valid, WASM well formed program constraint.
///
/// # Arguments
///
/// * 'ctx' - Z3 Context that this function is invoked
/// * 'instr_type' - The Instruction DatatypeSort
/// * 'instrs' - A slice of instruction Datatype to check validity
/// * 'acc' - An accumulator variable of type Int sort, representing the number of values on stack
///          before 'i'-th instruction.
/// * 'i' - the index of instruction in 'instrs' that this function needs process.
fn is_valid_helper<'a>(
    ctx: &'a Context,
    instr_type: &DatatypeSort<'a>,
    instrs: &[Datatype<'a>],
    acc: &Int<'a>,
    i: usize,
) -> Bool<'a> {
    // If we have checked all the instructions
    if i == instrs.len() {
        // Check the number of values left on the stack at the end.
        acc._eq(&Int::from_u64(&ctx, 1))
    } else {
        // Following lines compute essentially these.
        // if acc - pop < 0 {
        //   return false;
        // } else {
        //   is_valid_helper(acc - pop + push, i + 1);
        // }
        let pop = pop_cnts(&ctx, &instr_type, &instrs[i]);
        let res = acc.sub(&[&pop]);
        let push = push_cnts(&ctx, &instr_type, &instrs[i]);
        res.lt(&Int::from_u64(&ctx, 0)).ite(
            &Bool::from_bool(&ctx, false),
            &is_valid_helper(&ctx, &instr_type, &instrs, &res.add(&[&push]), i + 1),
        )
    }
}

// Well formed program constraint, given a slice of instructions.
fn is_valid<'a>(
    ctx: &'a Context,
    instr_type: &DatatypeSort<'a>,
    instrs: &[Datatype<'a>],
) -> Bool<'a> {
    let cnt = Int::from_u64(&ctx, 0);

    is_valid_helper(&ctx, &instr_type, &instrs, &cnt, 0)
}

// A function to convert a location variable to an instruction.
fn instr<'a>(ctx: &'a Context, instr_type: &DatatypeSort<'a>, lvar: &BV<'a>) -> Datatype<'a> {
    lvar._eq(&BV::from_u64(&ctx, 0, lvar.get_size())).ite(
        &instr_type.variants[0]
            .constructor
            .apply(&[])
            .as_datatype()
            .unwrap(),
        &lvar._eq(&BV::from_u64(&ctx, 1, lvar.get_size())).ite(
            &instr_type.variants[1]
                .constructor
                .apply(&[])
                .as_datatype()
                .unwrap(),
            &instr_type.variants[2]
                .constructor
                .apply(&[])
                .as_datatype()
                .unwrap(),
        ),
    )
}

fn instr_simplified<'a>(instr_type: &DatatypeSort<'a>, lvar: &BV<'a>) -> Datatype<'a> {
    // This line throws an exception because lvar is never instantiated to a concrete value, as we
    // want to find a concrete value that satisfies the constraint.
    let lvar = lvar.as_u64().unwrap();

    instr_type.variants[lvar as usize]
        .constructor
        .apply(&[])
        .as_datatype()
        .unwrap()
}

fn main() {
    let _ = env_logger::try_init();
    let cfg = Config::new();
    let ctx = Context::new(&cfg);

    // Define a new datatype named Instruction which has 3 variants.
    // 1. LocalGet 0
    // 2. I32Add
    // 3. Nop
    let instr_type = DatatypeBuilder::new(&ctx)
        .variant("LocalGet0", &[])
        .variant("I32Add", &[])
        .variant("Nop", &[])
        .finish("Instruction");

    let bit_width = (instr_type.variants.len() as f64).log(2.0).ceil() as u32;
    assert_eq!(bit_width, 2);

    let l0 = BV::new_const(&ctx, "l0", bit_width);
    let l1 = BV::new_const(&ctx, "l1", bit_width);
    let l2 = BV::new_const(&ctx, "l2", bit_width);

    let instrs = vec![
        instr(&ctx, &instr_type, &l0),
        instr(&ctx, &instr_type, &l1),
        instr(&ctx, &instr_type, &l2),
    ];

    let formula = is_valid(&ctx, &instr_type, &instrs);
    // Also needs to check the evaluation of this instruction sequence for inputs are the same with
    // respect to the spec.
    println!("{:?}", formula);

    let solver = Solver::new(&ctx);
    solver.assert(&formula);

    assert_eq!(solver.check(), SatResult::Sat);

    let model = solver.get_model();

    // An example output of this could be
    // 2
    // 2
    // 0
    // Which corresponds to
    // Nop
    // Nop
    // local.get 0
    // Which is a valid sequence of WASM instructions of length 3 given a function signature with
    // a parameter, for example f(x).
    println!("{}", model.eval(&l0).unwrap().as_u64().unwrap());
    println!("{}", model.eval(&l1).unwrap().as_u64().unwrap());
    println!("{}", model.eval(&l2).unwrap().as_u64().unwrap());

    // Create a new variable and try convert it to an instruction datatype using instr_simplified
    let l3 = BV::new_const(&ctx, "l3", bit_width);
    let result = std::panic::catch_unwind(|| {
        // This will throw a panic because l3 is just a variable and instr_simplified tries to
        // convert it to a concrete u64 value.
        instr_simplified(&instr_type, &l3);
    });
    assert!(result.is_err());

    // Just to check whether the catch_unwind above really works.
    println!("Run completed");
}
