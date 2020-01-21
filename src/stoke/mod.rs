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

    /// Finds a module that has functions equivalent to the functions in the given module.
    pub fn synthesize(&self) {
        // for func in module.functions {
        //   let test_cases = generate_test_cases(func)
        //   let candidate = generate_random_func(func.type);
        //     while (!candidate.validate()) {
        //       do_transform(candidate)
        //       if !exec_test_cases(func, test_cases) {
        //          continue
        //       }
        //       match verify(candidate, func) {
        //          Verified => break,
        //          CounterExample(inputs) => {
        //              let exepected_output = func.invoke_with(inputs);
        //              test_cases.push((inputs, output))
        //          }
        //       }
        //    }
        // }
    }
}
