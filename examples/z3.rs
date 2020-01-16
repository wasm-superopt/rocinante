extern crate env_logger;
extern crate log;
extern crate z3;

use z3::{
    ast::{forall_const, Array, Ast, Bool, Datatype, Dynamic, Int, BV},
    Config, Context, DatatypeBuilder, SatResult, Solver, Sort,
};

fn datatype() {
    let cfg = Config::new();
    let ctx = Context::new(&cfg);
    let solver = Solver::new(&ctx);
    // Like Rust's Option<int> type
    let option_int = DatatypeBuilder::new(&ctx)
        .variant("None", &[])
        .variant("Some", &[("value", &Sort::int(&ctx))])
        .finish("OptionInt");

    // Assert x.is_none()
    let x = Datatype::new_const(&ctx, "x", &option_int.sort);
    solver.assert(
        &option_int.variants[0]
            .tester
            .apply(&[&x.into()])
            .as_bool()
            .unwrap(),
    );

    // Assert y == Some(3)
    let y = Datatype::new_const(&ctx, "y", &option_int.sort);
    let value = option_int.variants[1]
        .constructor
        .apply(&[&Int::from_i64(&ctx, 3).into()]);
    solver.assert(&y._eq(&value.as_datatype().unwrap()));

    assert_eq!(solver.check(), SatResult::Sat);
    let model = solver.get_model();

    // Get the value out of Some(3)
    let ast = option_int.variants[1].accessors[0].apply(&[&y.into()]);
    assert_eq!(
        3,
        model
            .eval(&ast.as_int().unwrap())
            .unwrap()
            .as_i64()
            .unwrap()
    );
}

fn simple() {
    let _ = env_logger::try_init();
    let cfg = Config::new();
    let ctx = Context::new(&cfg);
    let x = Int::new_const(&ctx, "x");
    let y = Int::new_const(&ctx, "y");

    let solver = Solver::new(&ctx);
    solver.assert(&x.gt(&y));
    assert_eq!(solver.check(), SatResult::Sat);
}

fn shift() {
    let _ = env_logger::try_init();
    let cfg = Config::new();
    let ctx = Context::new(&cfg);
    let x = BV::new_const(&ctx, "x", 32);
    let one = BV::from_i64(&ctx, 1, 32);
    let two = BV::from_i64(&ctx, 2, 32);

    let forall = forall_const(
        &ctx,
        &[x.clone().into()],
        &[],
        &x.bvshl(&one).bvshl(&one)._eq(&x.bvshl(&two)).into(),
    )
    .as_bool()
    .unwrap();

    let solver = Solver::new(&ctx);
    solver.assert(&forall);

    assert_eq!(solver.check(), SatResult::Sat);
}

fn shift_int() {
    // Z3 doesn't support shl on ints. Convert it to BV.
    let _ = env_logger::try_init();
    let cfg = Config::new();
    let ctx = Context::new(&cfg);
    let x = Int::new_const(&ctx, "x");
    let one = BV::from_i64(&ctx, 1, 32);
    let two = BV::from_i64(&ctx, 2, 32);

    let forall = forall_const(
        &ctx,
        &[x.clone().into()],
        &[],
        &BV::from_int(&x, 32)
            .bvshl(&one)
            .bvshl(&one)
            ._eq(&BV::from_int(&x, 32).bvshl(&two))
            .into(),
    )
    .as_bool()
    .unwrap();
    let solver = Solver::new(&ctx);
    solver.assert(&forall);

    assert_eq!(solver.check(), SatResult::Sat);
}

fn add() {
    let _ = env_logger::try_init();
    let cfg = Config::new();
    let ctx = Context::new(&cfg);
    let x = Int::new_const(&ctx, "x");
    let three = Int::from_i64(&ctx, 3);
    let solver = Solver::new(&ctx);
    solver.assert(&x.add(&[&x])._eq(&x.mul(&[&three])));

    assert_eq!(solver.check(), SatResult::Sat);

    let model = solver.get_model();

    assert_eq!(0, model.eval(&x).unwrap().as_i64().unwrap());

    let two = Int::from_i64(&ctx, 2);

    let forall: Bool = forall_const(
        &ctx,
        &[x.clone().into()],
        &[],
        &x.add(&[&x])._eq(&x.mul(&[&two])).into(),
    )
    .as_bool()
    .unwrap();

    solver.assert(&forall);

    assert_eq!(solver.check(), SatResult::Sat);
}

fn local_init() {
    let _ = env_logger::try_init();
    let cfg = Config::new();
    let ctx = Context::new(&cfg);

    // Each parameter has a type explicitly declared.
    let local_type = DatatypeBuilder::new(&ctx)
        .variant("i32", &[("value", &Sort::int(&ctx))])
        .variant("i64", &[("value", &Sort::int(&ctx))])
        .variant("f32", &[("value", &Sort::int(&ctx))])
        .variant("f64", &[("value", &Sort::int(&ctx))])
        .finish("LocalType");

    // Create an array of locals.
    let locals = Array::new_const(&ctx, "locals", &Sort::int(&ctx), &local_type.sort);
    // Param 0
    locals.store(
        &Int::from_i64(&ctx, 0).into(),
        &local_type.variants[0]
            .constructor
            .apply(&[&Int::from_i64(&ctx, 2).into()]),
    );
}

fn mimick_stack() {
    let _ = env_logger::try_init();
    let cfg = Config::new();
    let ctx = Context::new(&cfg);
    let solver = Solver::new(&ctx);

    let mut vs_stack = Vec::new();

    let x = Int::new_const(&ctx, "x");
    vs_stack.push(Dynamic::from_ast(&x));

    vs_stack.push(Dynamic::from_ast(&Int::from_i64(&ctx, 2)));

    let lhs = vs_stack.pop().unwrap();
    let rhs = vs_stack.pop().unwrap();

    // Assuming that we're dealing with i32.mul
    let mul_res = lhs.as_int().unwrap().mul(&[&rhs.as_int().unwrap()]);
    vs_stack.push(Dynamic::from_ast(&mul_res));

    let f1_res = vs_stack.pop().unwrap().as_int().unwrap();

    let f2_res = x.add(&[&x]);

    let forall = forall_const(&ctx, &[x.clone().into()], &[], &f1_res._eq(&f2_res).into())
        .as_bool()
        .unwrap();

    solver.assert(&forall);
    assert_eq!(solver.check(), SatResult::Sat);
}

fn synthesize() {
    let _ = env_logger::try_init();
    let cfg = Config::new();
    let ctx = Context::new(&cfg);
    let x = Int::new_const(&ctx, "x");
    let c = Int::new_const(&ctx, "c");

    let forall: Bool = forall_const(
        &ctx,
        &[x.clone().into()],
        &[],
        &x.add(&[&x])._eq(&x.mul(&[&c])).into(),
    )
    .as_bool()
    .unwrap();

    let solver = Solver::new(&ctx);
    solver.assert(&forall);

    assert_eq!(solver.check(), SatResult::Sat);

    let model = solver.get_model();

    assert_eq!(2, model.eval(&c).unwrap().as_i64().unwrap());
}

fn main() {
    simple();
    datatype();
    shift();
    shift_int();
    add();
    local_init();
    mimick_stack();
    synthesize();
}
