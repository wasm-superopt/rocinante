use crate::utils;
use rand::Rng;
use wasmi::{nan_preserving_float, Error, ModuleInstance, NopExternals, RuntimeValue, ValueType};

const NUM_TEST_CASES: usize = 10;

#[derive(Clone, Debug)]
pub struct Input(Vec<RuntimeValue>);

impl Input {
    pub fn new(elements: Vec<RuntimeValue>) -> Self {
        Input(elements)
    }
    pub fn elements(&self) -> &[RuntimeValue] {
        &self.0
    }
}

#[derive(Debug)]
pub struct Output(Result<Option<RuntimeValue>, Error>);

impl Output {
    pub fn new(result: Result<Option<RuntimeValue>, Error>) -> Self {
        Output(result)
    }
    pub fn result(&self) -> &Result<Option<RuntimeValue>, Error> {
        &self.0
    }
}

#[derive(Debug)]
pub struct TestCases(Vec<(Input, Output)>);

impl TestCases {
    pub fn new(elements: Vec<(Input, Output)>) -> Self {
        TestCases(elements)
    }
    pub fn elements(&self) -> &[(Input, Output)] {
        &self.0
    }
    pub fn add_element(&mut self, test_case: (Input, Output)) {
        self.0.push(test_case)
    }
}

fn gen_random_input<R: Rng>(rng: &mut R, param_types: &[ValueType]) -> Input {
    let mut args: Vec<RuntimeValue> = Vec::with_capacity(param_types.len());

    for param_type in param_types {
        let arg = match param_type {
            ValueType::I32 => RuntimeValue::I32(rng.gen::<i32>()),
            ValueType::I64 => RuntimeValue::I64(rng.gen::<i64>()),
            ValueType::F32 => {
                RuntimeValue::F32(nan_preserving_float::F32::from_float(rng.gen::<f32>()))
            }
            ValueType::F64 => {
                RuntimeValue::F64(nan_preserving_float::F64::from_float(rng.gen::<f64>()))
            }
        };
        args.push(arg);
    }

    Input::new(args)
}

pub fn generate_test_cases<R: Rng>(
    rng: &mut R,
    instance: &ModuleInstance,
    func_name: &str,
) -> TestCases {
    let func = utils::func_by_name(instance, func_name).unwrap();
    let signature = func.signature();

    let mut inputs: Vec<Input> = Vec::with_capacity(NUM_TEST_CASES);
    for _ in 0..NUM_TEST_CASES {
        inputs.push(gen_random_input(rng, signature.params()));
    }

    let mut test_cases: Vec<(Input, Output)> = Vec::new();

    for input in inputs {
        let output =
            Output::new(instance.invoke_export(func_name, input.elements(), &mut NopExternals));

        test_cases.push((input, output));
    }

    TestCases::new(test_cases)
}
