use crate::utils;
use rand::Rng;
use wasmi::{nan_preserving_float, Error, ModuleInstance, NopExternals, RuntimeValue, ValueType};

const NUM_TEST_CASES: usize = 10;

type Input = Vec<RuntimeValue>;

type Output = Result<Option<RuntimeValue>, Error>;

type TestCases = Vec<(Input, Output)>;

fn gen_random_input<R: Rng>(rng: &mut R, param_types: &[ValueType]) -> Input {
    let mut inputs = Vec::with_capacity(param_types.len());

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
        inputs.push(arg);
    }

    inputs
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

    invoke_with_inputs(instance, func_name, &inputs)
}

pub fn invoke_with_inputs(
    instance: &ModuleInstance,
    func_name: &str,
    inputs: &[Input],
) -> TestCases {
    let mut test_cases: Vec<(Input, Output)> = Vec::new();
    for input in inputs {
        let output = instance.invoke_export(func_name, input, &mut NopExternals);
        test_cases.push((input.clone(), output));
    }
    test_cases
}

#[cfg(test)]
mod tests {
    use super::*;
    use wasmi::ImportsBuilder;

    #[test]
    fn test_invoke() {
        let wasm_binary: Vec<u8> = wabt::wat2wasm(
            r#"(module
                (type $t0 (func (param i32) (result i32)))
                (func $div (type $t0) (param $p0 i32) (result i32)
                  i32.const 4
                  get_local $p0
                  i32.div_u)
                (export "div" (func $div)))"#,
        )
        .expect("failed to parse wat");

        // Load wasm binary and prepare it for instantiation.
        let module = wasmi::Module::from_buffer(&wasm_binary).expect("failed to load wasm");

        // Instantiate a module with empty imports and
        // assert that there is no `start` function.
        let instance = ModuleInstance::new(&module, &ImportsBuilder::default())
            .expect("failed to instantiate wasm module")
            .assert_no_start();

        let expected_input: Vec<Input> =
            vec![vec![RuntimeValue::I32(2)], vec![RuntimeValue::I32(0)]];

        let expected_output: Vec<Output> = vec![
            Result::Ok(Some(RuntimeValue::I32(2))),
            Result::Err(Error::Trap(wasmi::Trap::new(
                wasmi::TrapKind::DivisionByZero,
            ))),
        ];

        let test_cases = invoke_with_inputs(&instance, "div", &expected_input);

        assert_eq!(expected_input.len(), test_cases.len());
        for i in 0..expected_input.len() {
            let expected_input = &expected_input[i];
            let expected_output = &expected_output[i];

            let (actual_input, actual_output) = &test_cases[i];

            assert_eq!(expected_input, actual_input);

            if expected_output.is_err() {
                // wasmi::Error doesn't implement PartiqlEq and can't directly
                // be tested for equality, so conver to String.
                assert_eq!(
                    expected_output.as_ref().err().unwrap().to_string(),
                    actual_output.as_ref().err().unwrap().to_string(),
                );
            } else {
                assert_eq!(
                    expected_output.as_ref().ok().unwrap(),
                    actual_output.as_ref().ok().unwrap(),
                );
            }
        }
    }
}
