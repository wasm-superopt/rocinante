extern crate z3;

use parity_wasm::elements::{FuncBody, FunctionType, Instruction, Instructions, Local, ValueType};
use z3::ast::Ast;
use z3::{ast, Context, SatResult, Solver};

fn locals_to_z3_vec<'ctx>(ctx: &'ctx Context, locals: &[Local]) -> Vec<ast::Int<'ctx>> {
    let mut z3_locals: Vec<ast::Int<'ctx>> = Vec::with_capacity(locals.len());
    for local in locals {
        let cnt = local.count();
        let local_type = local.value_type();

        for _ in 0..cnt {
            match local_type {
                ValueType::I32 => {
                    // The initial value of any local is 0.
                    z3_locals.push(ast::Int::from_i64(&ctx, 0));
                }
                _ => {
                    assert!(false, "{} not supported", local_type);
                }
            }
        }
    }
    z3_locals
}

fn get_local<'ctx>(
    params: &'ctx [ast::Int<'ctx>],
    locals: &'ctx [ast::Int<'ctx>],
    idx: u32,
) -> Option<&'ctx ast::Int<'ctx>> {
    let num_params = params.len() as u32;
    let num_locals = locals.len() as u32;

    if idx < num_params {
        Some(&params[idx as usize])
    } else if idx - num_params < num_locals {
        Some(&locals[(idx - num_params) as usize])
    } else {
        None
    }
}

fn to_z3_formula<'ctx>(
    ctx: &'ctx Context,
    params: &'ctx [ast::Int<'ctx>],
    locals: &'ctx [ast::Int<'ctx>],
    _return_type: Option<ValueType>,
    instrs: &Instructions,
    stack: &mut Vec<ast::Int<'ctx>>,
) {
    for (_i, instr) in instrs.elements().iter().enumerate() {
        match instr {
            Instruction::I32Add => {
                let lhs: &ast::Int<'ctx> = &stack.pop().expect("stack empty");
                let rhs: &ast::Int<'ctx> = &stack.pop().expect("stack empty");
                let res = lhs.add(&[&rhs]);
                stack.push(res);
            }
            Instruction::I32Mul => {
                let lhs: &ast::Int<'ctx> = &stack.pop().expect("stack empty");
                let rhs: &ast::Int<'ctx> = &stack.pop().expect("stack empty");
                let res = lhs.mul(&[&rhs]);
                stack.push(res);
            }
            Instruction::I32Const(c) => {
                let val = ast::Int::from_i64(ctx, *c as i64);
                stack.push(val);
            }
            Instruction::GetLocal(idx) => {
                let val = get_local(params, locals, *idx)
                    .unwrap_or_else(|| {
                        panic!(
                            "Local index out of bounds: #params: {} #locals: {} idx: {}",
                            params.len(),
                            locals.len(),
                            idx
                        )
                    })
                    .clone();
                stack.push(val);
            }
            Instruction::End => {
                // We don't need to model this yet.
            }
            _ => {
                assert!(false, "{} not supported", instr);
            }
        }
    }
}

#[allow(dead_code)]
pub fn verify(func_type: &FunctionType, spec: &FuncBody, candidate: &FuncBody) -> SatResult {
    let cfg = z3::Config::new();
    let ctx = z3::Context::new(&cfg);

    // TODO(taegyunkim): Support for floats, and disambiguation between 32 and 64 bits.
    let mut shared_params: Vec<ast::Int> = Vec::with_capacity(func_type.params().len());
    for param in func_type.params() {
        match param {
            ValueType::I32 => {
                shared_params.push(ast::Int::fresh_const(&ctx, "p"));
            }
            param_type => {
                assert!(false, "{} not supported", param_type);
            }
        }
    }

    let spec_locals: Vec<ast::Int> = locals_to_z3_vec(&ctx, spec.locals());

    let candidate_locals: Vec<ast::Int> = locals_to_z3_vec(&ctx, candidate.locals());
    let mut spec_stack = Vec::new();
    to_z3_formula(
        &ctx,
        &shared_params,
        &spec_locals,
        func_type.return_type(),
        spec.code(),
        &mut spec_stack,
    );
    let spec_f = spec_stack.pop().expect("spec stack empty");

    let mut candidate_stack = Vec::new();
    to_z3_formula(
        &ctx,
        &shared_params,
        &candidate_locals,
        func_type.return_type(),
        candidate.code(),
        &mut candidate_stack,
    );
    let candidate_f = candidate_stack.pop().expect("candidate stack empty");

    let solver = Solver::new(&ctx);

    let bounds: Vec<ast::Dynamic> = shared_params.iter().map(|p| p.clone().into()).collect();

    let forall = ast::forall_const(&ctx, &bounds, &[], &spec_f._eq(&candidate_f).into())
        .as_bool()
        .unwrap();

    solver.assert(&forall);
    solver.check()
}
