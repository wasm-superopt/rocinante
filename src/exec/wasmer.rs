use super::{Interpreter, InterpreterKind, EPSILON, NUM_TEST_CASES};
use rand::Rng;
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

        let return_type = func.signature().params();
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

    fn eval_test_cases(&self, candidate: &[u8]) -> u32 {
        let return_type_bits: u32 = self.return_type_bits.iter().sum();

        let module_or_err = compile_with_config(
            candidate,
            CompilerConfig {
                enforce_stack_check: true,
                ..Default::default()
            },
        );
        if module_or_err.is_err() {
            return (return_type_bits + EPSILON) * self.test_cases.len() as u32;
        }
        let module = module_or_err.unwrap();
        let import_object = imports! {};
        let instance_or_err = module.instantiate(&import_object);
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

    fn add_test_case(&mut self, wasmi_input: &[::wasmi::RuntimeValue]) {
        let func = self.instance.dyn_func(&self.func_name).unwrap();

        let input: Vec<Value> = wasmi_input
            .iter()
            .map(|i| match i {
                ::wasmi::RuntimeValue::I32(x) => Value::I32(*x),
                unimplemented => panic!("type not implemented {:?}", unimplemented),
            })
            .collect();

        let output = func.call(&input);
        self.test_cases.push((input, output));
    }

    fn return_bit_width(&self) -> u32 {
        self.return_type_bits.iter().sum()
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

#[cfg(test)]
mod test {
    use super::*;
    #[test]
    fn sanity_test() {
        let binary =
            wat::parse_file("./examples/hackers_delight/p7.wat").expect("Failed to parse .wat");
        let mut interpreter = Wasmer::new(&binary, "p7");
        interpreter.add_test_case(&[::wasmi::RuntimeValue::I32(2147483647)]);

        let candidate = wabt::wat2wasm(
            r#"(module
                (func $p7 (export "p7") (param i32) (result i32)
                  i32.const -1
                  local.get 0
                  i32.const -2
                  local.get 0
                  i32.sub
                  i32.or
                  i32.rem_u
                )
              )"#,
        )
        .expect("Failed to convert to binary");

        let cost = interpreter.eval_test_cases(&candidate);
        println!("{}", cost);
        assert_ne!(cost, 0);
    }
}
