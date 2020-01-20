use parity_wasm::elements::Module;

pub mod transform;

#[allow(dead_code)]
pub struct Optimizer {
    module: Module,
}

impl Optimizer {
    pub fn new(module: Module) -> Self {
        Optimizer { module }
    }

    pub fn run(&self) {}
}
