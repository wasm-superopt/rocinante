extern crate env_logger;
extern crate log;
extern crate z3;

use z3::{
    ast::{forall_const, Array, Ast, Bool, Datatype, Dynamic, Int, BV},
    Config, Context, DatatypeBuilder, FuncDecl, Pattern, SatResult, Solver, Sort,
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

    let solver = Solver::new(&ctx);
    solver.assert(&x.bvshl(&one).bvshl(&one)._eq(&x.bvshl(&two)));

    assert_eq!(solver.check(), SatResult::Sat);
}

fn add() {
    let _ = env_logger::try_init();
    let cfg = Config::new();
    let ctx = Context::new(&cfg);
    let x = Int::new_const(&ctx, "x");
    let solver = Solver::new(&ctx);
    solver.assert(&x.add(&[&x])._eq(&x.mul(&[&Int::from_i64(&ctx, 2)])));

    assert_eq!(solver.check(), SatResult::Sat);
}

fn add_eq() {
    let _ = env_logger::try_init();
    let cfg = Config::new();
    let ctx = Context::new(&cfg);
    let x = Int::new_const(&ctx, "x");
    let solver = Solver::new(&ctx);

    let f1 = FuncDecl::new(&ctx, "f1", &[&Sort::int(&ctx)], &Sort::int(&ctx));
    let f1_x: z3::ast::Int = f1.apply(&[&x.clone().into()]).as_int().unwrap();
    let f1_x_pattern = Pattern::new(&ctx, &[&Dynamic::from_ast(&x.add(&[&x]))]);

    let f2 = FuncDecl::new(&ctx, "f2", &[&Sort::int(&ctx)], &Sort::int(&ctx));
    let f2_x: z3::ast::Int = f2.apply(&[&x.clone().into()]).as_int().unwrap();
    let f2_x_pattern = Pattern::new(
        &ctx,
        &[&Dynamic::from_ast(&x.mul(&[&Int::from_i64(&ctx, 2)]))],
    );

    let forall: Bool = forall_const(
        &ctx,
        &[&x.clone().into()],
        &[&f1_x_pattern, &f2_x_pattern],
        &f1_x._eq(&f2_x).into(),
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

fn main() {
    simple();
    datatype();
    shift();
    add();
    add_eq();
    local_init();
}
