use super::{Interpreter, InterpreterKind, EPSILON, NUM_TEST_CASES};
use rand::Rng;
use wasmer_runtime::*;

pub type Input = Vec<Value>;

pub type Output = Result<Vec<Value>, error::CallError>;

pub type TestCases = Vec<(Input, Output)>;

pub struct Wasmer {
    kind: InterpreterKind,
    test_cases: TestCases,
    return_type_bits: Vec<u32>,
}

impl Interpreter for Wasmer {
    fn new() -> Self {
        Self {
            kind: InterpreterKind::Wasmer,
            test_cases: Vec::new(),
            return_type_bits: Vec::new(),
        }
    }

    fn kind(&self) -> InterpreterKind {
        self.kind
    }

    fn print_test_cases(&self) {}

    fn generate_test_cases(&mut self, spec: &[u8], func_name: &str) {
        let import_object = imports! {};
        let instance = instantiate(spec, &import_object).unwrap();
        let func = instance.dyn_func(func_name).unwrap();
        let mut inputs: Vec<Input> = Vec::with_capacity(NUM_TEST_CASES);
        for _ in 0..NUM_TEST_CASES {
            inputs.push(gen_random_input(func.signature().params()));
        }
        let outputs = invoke_with_inputs(&func, &inputs);
        self.test_cases = inputs.into_iter().zip(outputs.into_iter()).collect();

        let return_type = func.signature().params();
        for typ in return_type {
            match typ {
                types::Type::I32 => {
                    self.return_type_bits.push(32);
                }
                unimplemented => {
                    panic!("{:?} type not implemented", unimplemented);
                }
            }
        }
    }

    fn eval_test_cases(&self, candidate: &[u8]) -> u32 {
        let return_type_bits: u32 = self.return_type_bits.iter().sum();

        let import_object = imports! {};
        let instance_or_err = instantiate(candidate, &import_object);
        if instance_or_err.is_err() {
            return (return_type_bits + EPSILON) * self.test_cases.len() as u32;
        }
        let instance = instance_or_err.unwrap();
        let func_or_err = instance.dyn_func("candidate");
        if func_or_err.is_err() {
            return (return_type_bits + EPSILON) * self.test_cases.len() as u32;
        }
        let func = func_or_err.unwrap();
        let mut dist = 0;
        for (input, expected_output) in &self.test_cases {
            let actual_output = func.call(&input);
            dist += hamming_distance(&expected_output, &actual_output);
        }
        dist
    }
}

fn hamming_distance(output1: &Output, output2: &Output) -> u32 {
    let mut dist = 0;

    match (output1, output2) {
        (Ok(val_vec1), Ok(val_vec2)) => {
            assert_eq!(
                val_vec1.len(),
                val_vec2.len(),
                "Spec and candidate function return type don't match."
            );

            for i in 0..val_vec1.len() {
                let val1 = &val_vec1[i];
                let val2 = &val_vec2[i];

                match (val1, val2) {
                    (Value::I32(x), Value::I32(y)) => {
                        dist += (x ^ y).count_ones();
                    }
                    _ => {
                        panic!("type not supported.");
                    }
                }
            }
        }
        (Ok(val_vec), Err(_)) | (Err(_), Ok(val_vec)) => {
            for val in val_vec {
                match val {
                    Value::I32(_) => dist += 32,
                    _ => panic!("type not supported."),
                }
            }
        }
        (Err(err1), Err(err2)) => {
            if err1 != err2 {
                dist += 32
            }
        }
    }

    dist
}

fn gen_random_input(param_types: &[types::Type]) -> Input {
    let mut inputs = Vec::with_capacity(param_types.len());
    for param_type in param_types {
        let arg = match param_type {
            types::Type::I32 => Value::I32(rand::thread_rng().gen::<i32>()),
            unexpected => {
                panic!("{:?} type not supported.", unexpected);
            }
        };
        inputs.push(arg);
    }

    inputs
}

fn invoke_with_inputs(func: &DynFunc, inputs: &[Input]) -> Vec<Output> {
    let mut outputs: Vec<Output> = Vec::with_capacity(inputs.len());
    for input in inputs {
        let output = func.call(input);
        outputs.push(output);
    }
    outputs
}
