extern crate wasmi;

use wasmi::{Error, ModuleInstance, NopExternals, RuntimeValue};

pub struct Input(Vec<RuntimeValue>);

impl Input {
    pub fn new(elements: Vec<RuntimeValue>) -> Self {
        Input(elements)
    }
    pub fn elements(&self) -> &[RuntimeValue] {
        &self.0
    }
}

pub struct Output(Result<Option<RuntimeValue>, Error>);

impl Output {
    pub fn new(result: Result<Option<RuntimeValue>, Error>) -> Self {
        Output(result)
    }
    pub fn result(&self) -> &Result<Option<RuntimeValue>, Error> {
        &self.0
    }
}

pub struct TestCases(Vec<(Input, Output)>);

impl TestCases {
    pub fn new(elements: Vec<(Input, Output)>) -> Self {
        TestCases(elements)
    }
    pub fn elements(&self) -> &[(Input, Output)] {
        &self.0
    }
}

pub fn generate_test_cases(instance: &ModuleInstance, func_name: &str) {
    let inputs: Vec<Input> = Vec::new();

    let mut test_cases: Vec<(Input, Output)> = Vec::new();

    for input in inputs {
        let output =
            Output::new(instance.invoke_export(func_name, input.elements(), &mut NopExternals));

        test_cases.push((input, output));
    }
}
