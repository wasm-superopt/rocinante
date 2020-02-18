use super::{Interpreter, InterpreterKind, EPSILON, NUM_TEST_CASES};
use rand::Rng;
use wasmtime::*;

pub type Input = Vec<Val>;

pub type Output = Result<Box<[Val]>, Trap>;

pub type TestCases = Vec<(Input, Output)>;

pub struct Wasmtime {
    store: Store,
    instance: Instance,
    func_name: String,
    test_cases: TestCases,
    return_type_bits: Vec<u32>,
}

impl Wasmtime {
    pub fn new(spec: &[u8], func_name: &str) -> Self {
        let store = wasmtime::Store::default();
        let module = Module::new(&store, &spec).unwrap();
        let instance = Instance::new(&store, &module, &[]).unwrap();
        let func = instance
            .find_export_by_name(func_name)
            .unwrap()
            .func()
            .unwrap()
            .borrow();

        let mut inputs: Vec<Input> = Vec::with_capacity(NUM_TEST_CASES);
        for _ in 0..NUM_TEST_CASES {
            inputs.push(gen_random_input(func.r#type().params()));
        }
        let outputs: Vec<Output> = inputs.iter().map(|input| func.call(input)).collect();
        let test_cases = inputs.into_iter().zip(outputs.into_iter()).collect();

        let return_type = func.r#type().results();
        let mut return_type_bits = Vec::new();
        for typ in return_type {
            match typ {
                ValType::I32 => {
                    return_type_bits.push(32);
                }
                unimplemented => {
                    panic!("{:?} type not implemented", unimplemented);
                }
            }
        }

        // TODO(taegyunkim): Instead of explicitly dropping the reference, just keep it by
        // specifying explicit lifetime.
        drop(func);
        Self {
            store,
            instance,
            func_name: String::from(func_name),
            test_cases,
            return_type_bits,
        }
    }
}

impl Interpreter for Wasmtime {
    fn kind(&self) -> InterpreterKind {
        InterpreterKind::Wasmtime
    }

    fn print_test_cases(&self) {}

    fn eval_test_cases(&self, candidate: &[u8]) -> u32 {
        let return_type_bits: u32 = self.return_type_bits.iter().sum();

        let module_or_err = Module::new(&self.store, &candidate);
        if module_or_err.is_err() {
            return (return_type_bits + EPSILON) * self.test_cases.len() as u32;
        }
        let module = module_or_err.unwrap();
        let instance_or_err = Instance::new(&self.store, &module, &[]);
        if instance_or_err.is_err() {
            return (return_type_bits + EPSILON) * self.test_cases.len() as u32;
        }
        let instance = instance_or_err.unwrap();
        let func = instance
            .find_export_by_name("candidate")
            .expect("Export with name candidate doesn't exist, should never happen.")
            .func()
            .expect("Export candidate is not a function, should never happen.")
            .borrow();
        let mut dist = 0;
        for (input, expected_output) in &self.test_cases {
            let actual_output = func.call(&input);
            dist += hamming_distance(&expected_output, &actual_output);
        }
        dist
    }

    fn add_test_case(&mut self, wasmi_input: &[::wasmi::RuntimeValue]) {
        let func = self
            .instance
            .find_export_by_name(&self.func_name)
            .unwrap()
            .func()
            .unwrap()
            .borrow();

        let new_input: Vec<Val> = wasmi_input
            .iter()
            .map(|i| match i {
                ::wasmi::RuntimeValue::I32(x) => Val::I32(*x),
                unimplemented => panic!("type not implemented {:?}", unimplemented),
            })
            .collect();

        let output = func.call(&new_input);
        self.test_cases.push((new_input, output));
    }

    fn return_bit_width(&self) -> u32 {
        self.return_type_bits.iter().sum()
    }
}

fn gen_random_input(param_types: &[ValType]) -> Input {
    let mut input = Vec::with_capacity(param_types.len());
    for param_type in param_types {
        let arg = match param_type {
            ValType::I32 => Val::I32(rand::thread_rng().gen::<i32>()),
            unimplemented => {
                panic!("{:?} type not implemented.", unimplemented);
            }
        };
        input.push(arg);
    }
    input
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
                    (Val::I32(x), Val::I32(y)) => {
                        dist += (x ^ y).count_ones();
                    }
                    unimplemented => {
                        panic!("{:?} type not supported.", unimplemented);
                    }
                }
            }
        }
        (Ok(val_vec), Err(_)) | (Err(_), Ok(val_vec)) => {
            for val in val_vec.iter() {
                match val {
                    Val::I32(_) => dist += 32,
                    _ => panic!("type not supported."),
                }
            }
        }
        (Err(_), Err(_)) => {
            // TODO(taegyunkim): Figure out a right way to compare traps.
        }
    }
    dist
}
