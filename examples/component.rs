extern crate env_logger;
extern crate log;
extern crate z3;

use z3::{
    ast::{Ast, Bool, Datatype, Int, BV},
    Config, Context, DatatypeBuilder, DatatypeSort, SatResult, Solver,
};

fn main() {
    let _ = env_logger::try_init();
    let cfg = Config::new();
    let ctx = Context::new(&cfg);

    let components = DatatypeBuilder::new(&ctx)
        .variant("LocalGet0", &[])
        .variant("I32Add", &[])
        .variant("Nop", &[])
        .finish("Instruction");

    fn pop_cnts<'a>(
        ctx: &'a Context,
        components: &DatatypeSort<'a>,
        instr: &Datatype<'a>,
    ) -> Int<'a> {
        components.variants[0]
            .tester
            .apply(&[&instr.clone().into()])
            .as_bool()
            .unwrap()
            .ite(
                &(Int::from_u64(&ctx, 0)),
                &components.variants[1]
                    .tester
                    .apply(&[&instr.clone().into()])
                    .as_bool()
                    .unwrap()
                    .ite(&(Int::from_u64(&ctx, 2)), &(Int::from_u64(&ctx, 0))),
            )
    }

    fn push_cnts<'a>(
        ctx: &'a Context,
        components: &DatatypeSort<'a>,
        instr: &Datatype<'a>,
    ) -> Int<'a> {
        components.variants[0]
            .tester
            .apply(&[&instr.clone().into()])
            .as_bool()
            .unwrap()
            .ite(
                &(Int::from_u64(&ctx, 1)),
                &components.variants[1]
                    .tester
                    .apply(&[&instr.clone().into()])
                    .as_bool()
                    .unwrap()
                    .ite(&(Int::from_u64(&ctx, 1)), &(Int::from_u64(&ctx, 0))),
            )
    }

    fn is_valid<'a>(
        ctx: &'a Context,
        components: &DatatypeSort<'a>,
        instrs: &[Datatype<'a>],
    ) -> Bool<'a> {
        let cnt = Int::from_u64(&ctx, 0);
        let zero = Int::from_u64(&ctx, 0);

        fn is_valid_helper<'a>(
            ctx: &'a Context,
            components: &DatatypeSort<'a>,
            instrs: &[Datatype<'a>],
            zero: &Int<'a>,
            acc: &Int<'a>,
            i: usize,
        ) -> Bool<'a> {
            if i == instrs.len() {
                acc._eq(&Int::from_u64(&ctx, 1))
            } else {
                let pop = pop_cnts(&ctx, &components, &instrs[i]);
                let res = acc.sub(&[&pop]);
                let push = push_cnts(&ctx, &components, &instrs[i]);
                res.lt(&zero).ite(
                    &Bool::from_bool(&ctx, false),
                    &is_valid_helper(&ctx, &components, &instrs, &zero, &res.add(&[&push]), i + 1),
                )
            }
        }

        is_valid_helper(&ctx, &components, &instrs, &zero, &cnt, 0)
    }

    let bit_width = (components.variants.len() as f64).log(2.0).ceil() as u32;
    assert_eq!(bit_width, 2);

    fn instr<'a>(ctx: &'a Context, components: &DatatypeSort<'a>, lvar: &BV<'a>) -> Datatype<'a> {
        lvar._eq(&BV::from_u64(&ctx, 0, lvar.get_size())).ite(
            &components.variants[0]
                .constructor
                .apply(&[])
                .as_datatype()
                .unwrap(),
            &lvar._eq(&BV::from_u64(&ctx, 1, lvar.get_size())).ite(
                &components.variants[1]
                    .constructor
                    .apply(&[])
                    .as_datatype()
                    .unwrap(),
                &components.variants[2]
                    .constructor
                    .apply(&[])
                    .as_datatype()
                    .unwrap(),
            ),
        )
    }

    let l0 = BV::new_const(&ctx, "l0", bit_width);
    let l1 = BV::new_const(&ctx, "l1", bit_width);
    let l2 = BV::new_const(&ctx, "l2", bit_width);

    let instrs = vec![
        instr(&ctx, &components, &l0),
        instr(&ctx, &components, &l1),
        instr(&ctx, &components, &l2),
    ];

    let formula = is_valid(&ctx, &components, &instrs);
    println!("{:?}", formula);

    let solver = Solver::new(&ctx);
    solver.assert(&formula);

    assert_eq!(solver.check(), SatResult::Sat);

    let model = solver.get_model();

    println!("{}", model.eval(&l0).unwrap().as_u64().unwrap());
    println!("{}", model.eval(&l1).unwrap().as_u64().unwrap());
    println!("{}", model.eval(&l2).unwrap().as_u64().unwrap());
}
