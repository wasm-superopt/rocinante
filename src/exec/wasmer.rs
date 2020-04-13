use super::{Interpreter, InterpreterKind, NUM_TEST_CASES};
use rand::Rng;
use wasmer_runtime::error::CallResult;
use wasmer_runtime::*;

pub type Input = Vec<Value>;

pub type Output = Result<Vec<Value>, error::CallError>;

pub type TestCases = Vec<(Input, Output)>;

pub struct Wasmer {
    instance: Instance,
    func_name: String,
    test_cases: TestCases,
    return_type_bits: Vec<u32>,
}

impl Wasmer {
    pub fn new(spec: &[u8], func_name: &str) -> Self {
        let import_object = imports! {};
        let instance = instantiate(spec, &import_object).unwrap();
        let func = instance.dyn_func(func_name).unwrap();
        let mut inputs: Vec<Input> = Vec::with_capacity(NUM_TEST_CASES);
        for _ in 0..NUM_TEST_CASES {
            inputs.push(gen_random_input(func.signature().params()));
        }
        let outputs: Vec<Output> = inputs.iter().map(|input| func.call(input)).collect();
        let test_cases = inputs.into_iter().zip(outputs.into_iter()).collect();

        let return_type = func.signature().returns();
        assert_eq!(1, return_type.len(), "Doesn't support multi-value returns.");
        let mut return_type_bits = Vec::new();
        for typ in return_type {
            match typ {
                types::Type::I32 => {
                    return_type_bits.push(32);
                }
                unimplemented => {
                    panic!("{:?} type not implemented", unimplemented);
                }
            }
        }

        Self {
            instance,
            func_name: String::from(func_name),
            test_cases,
            return_type_bits,
        }
    }
}

impl Interpreter for Wasmer {
    fn kind(&self) -> InterpreterKind {
        InterpreterKind::Wasmer
    }

    fn print_test_cases(&self) {}

    fn eval_test_cases(&self, binary: &[u8]) -> u32 {
        let import_object = imports! {};
        let instance_or_err = instantiate(binary, &import_object);
        let instance = instance_or_err.unwrap();
        let func_or_err = instance.dyn_func("candidate");
        let func = func_or_err.unwrap();
        let mut dist = 0;
        for (input, expected_output) in &self.test_cases {
            let actual_output = func.call(&input);
            dist += hamming_distance(&expected_output, &actual_output);
        }
        dist
    }

    fn get_test_outputs(&self, binary: &[u8]) -> Vec<CallResult<Vec<Value>>> {
        let import_object = imports! {};
        let instance_or_err = instantiate(binary, &import_object);
        let instance = instance_or_err.unwrap();
        let func_or_err = instance.dyn_func("candidate");
        let func = func_or_err.unwrap();

        let mut diffs = Vec::new();
        for (input, expected_output) in &self.test_cases {
            let actual_output = func.call(&input);
            if *expected_output != actual_output {
                diffs.push(actual_output);
            }
        }

        diffs
    }

    fn add_test_case(&mut self, input: Vec<::wasmer_runtime::Value>) {
        let func = self.instance.dyn_func(&self.func_name).unwrap();
        let output = func.call(&input);
        self.test_cases.push((input, output));
    }

    fn return_type_len(&self) -> usize {
        self.return_type_bits.len()
    }

    fn return_bit_width(&self) -> u32 {
        self.return_type_bits.iter().sum()
    }

    fn num_test_cases(&self) -> usize {
        self.test_cases.len()
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

            for (val1, val2) in val_vec1.iter().zip(val_vec2.iter()) {
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
