use z3::{ast, Context};

pub struct Converter<'ctx> {
    ctx: &'ctx Context,
    params: Vec<ast::Dynamic<'ctx>>,
    _return_type: Option<parity_wasm::elements::ValueType>,
}

impl<'ctx> Converter<'ctx> {
    pub fn new(ctx: &'ctx Context, func_type: &parity_wasm::elements::FunctionType) -> Self {
        let mut params: Vec<ast::Dynamic<'ctx>> = Vec::with_capacity(func_type.params().len());

        for param in func_type.params() {
            match param {
                parity_wasm::elements::ValueType::I32 => {
                    params.push(ast::BV::fresh_const(&ctx, "p", 32).into())
                }
                parity_wasm::elements::ValueType::I64 => {
                    params.push(ast::BV::fresh_const(&ctx, "p", 64).into())
                }
                _ => {
                    panic!("float not supported.");
                }
            }
        }

        Self {
            ctx,
            params,
            _return_type: func_type.return_type(),
        }
    }

    fn init_locals(&self, local_types: &[parity_wasm::elements::Local]) -> Vec<ast::Dynamic<'ctx>> {
        let mut locals = self.params.clone();
        locals.reserve(local_types.len());

        for local in local_types {
            let cnt = local.count();
            let local_type = local.value_type();

            for _ in 0..cnt {
                match local_type {
                    // Initial value of any local is 0.
                    parity_wasm::elements::ValueType::I32 => {
                        locals.push(ast::BV::from_u64(&self.ctx, 0, 32).into())
                    }
                    parity_wasm::elements::ValueType::I64 => {
                        locals.push(ast::BV::from_u64(&self.ctx, 0, 64).into())
                    }
                    _ => {
                        panic!("float not supported.");
                    }
                }
            }
        }

        locals
    }

    pub fn convert_func(&self, func: &parity_wasm::elements::FuncBody) {
        let mut _locals = self.init_locals(func.locals());
        let mut _stack: Vec<ast::Dynamic<'ctx>> = Vec::new();
    }
}
