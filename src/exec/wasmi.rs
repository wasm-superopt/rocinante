use crate::wasmi_utils;
use rand::Rng;
use wasmi::{
    nan_preserving_float, FuncInstance, FuncRef, NopExternals, RuntimeValue, Trap, ValueType,
};

const NUM_TEST_CASES: usize = 10;

pub type Input = Vec<RuntimeValue>;

pub type Output = Result<Option<RuntimeValue>, Trap>;

/// Computes hamming distance between the outputs. We intentinoally use u32 here
/// as the return type as it doesn't make sense to have negative values for the
/// hamming distance.
pub fn hamming_distance(output1: &Output, output2: &Output) -> u32 {
    match (output1, output2) {
        (Ok(val_opt1), Ok(val_opt2)) => match (val_opt1, val_opt2) {
            (None, None) => panic!("Doens't support void functions."),
            (Some(val1), Some(val2)) => match (val1, val2) {
                (RuntimeValue::I32(x), RuntimeValue::I32(y)) => (x ^ y).count_ones(),
                (RuntimeValue::I64(x), RuntimeValue::I64(y)) => (x ^ y).count_ones(),
                (RuntimeValue::F32(x), RuntimeValue::F32(y)) => {
                    (x.to_bits() ^ y.to_bits()).count_ones()
                }
                (RuntimeValue::F64(x), RuntimeValue::F64(y)) => {
                    (x.to_bits() ^ y.to_bits()).count_ones()
                }
                _ => panic!("Spec and candidate function return type don't match."),
            },
            _ => panic!("Spec and candidate function return type don't match."),
        },
        (Ok(val_opt), Err(_)) => match val_opt {
            None => panic!("doesn't support void functions."),
            Some(val) => match val {
                RuntimeValue::I32(_) => 32,
                RuntimeValue::I64(_) => 64,
                RuntimeValue::F32(_) => 32,
                RuntimeValue::F64(_) => 64,
            },
        },
        (Err(_), Ok(val_opt)) => match val_opt {
            None => panic!("doesn't support void functions."),
            Some(val) => match val {
                RuntimeValue::I32(_) => 32,
                RuntimeValue::I64(_) => 64,
                RuntimeValue::F32(_) => 32,
                RuntimeValue::F64(_) => 64,
            },
        },
        // TODO(taegyunkim): Use sensible values for errors.
        (Err(err1), Err(err2)) => {
            if err1.to_string() == err2.to_string() {
                0
            } else {
                32
            }
        }
    }
}

pub type TestCases = Vec<(Input, Output)>;

fn gen_random_input<R: Rng + ?Sized>(rng: &mut R, param_types: &[ValueType]) -> Input {
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

pub fn generate_test_cases<R: Rng + ?Sized>(
    rng: &mut R,
    instance: &wasmi::ModuleInstance,
    func_name: &str,
) -> TestCases {
    let func_ref = wasmi_utils::func_by_name(instance, func_name)
        .unwrap_or_else(|_| panic!("Module doesn't have function named {}", func_name));
    let signature = func_ref.signature();

    let mut inputs: Vec<Input> = Vec::with_capacity(NUM_TEST_CASES);
    for _ in 0..NUM_TEST_CASES {
        inputs.push(gen_random_input(rng, signature.params()));
    }

    let outputs = invoke_with_inputs(&func_ref, &inputs);

    inputs.into_iter().zip(outputs.into_iter()).collect()
}

pub fn invoke_with_inputs(func_ref: &FuncRef, inputs: &[Input]) -> Vec<Output> {
    let mut outputs: Vec<Output> = Vec::with_capacity(inputs.len());
    for input in inputs {
        let output = FuncInstance::invoke(func_ref, input, &mut NopExternals);
        outputs.push(output);
    }
    outputs
}

// NOTE(taegyunkim): When a given WASM module isn't valid, wasmi crate panics when we
// try to instantiate it via a call to wasmi::Module::from_parity_wasm_module(),
// and outputs error message unnecessarily. This is to suppress that.
// https://stackoverflow.com/a/59211505
fn catch_unwind_silent<F: FnOnce() -> R + std::panic::UnwindSafe, R>(
    f: F,
) -> std::thread::Result<R> {
    let prev_hook = std::panic::take_hook();
    std::panic::set_hook(Box::new(|_| {}));
    let result = std::panic::catch_unwind(f);
    std::panic::set_hook(prev_hook);
    result
}

// NOTE(taegyunkim): The return type of this function is unsigned instead of
// signed because it represents the sum of hamming distances. When it overflows,
// rust will panic.
pub fn eval_test_cases(
    module: &parity_wasm::elements::Module,
    test_cases: &[(Input, Output)],
) -> u32 {
    // The module is validated this step.
    let result_or_err =
        catch_unwind_silent(|| wasmi::Module::from_parity_wasm_module(module.clone()));
    if result_or_err.is_err() {
        #[cfg(debug_assertions)]
        println!("Failed to convert to wasmi module.");
        return 64 * test_cases.len() as u32;
    }

    let module_or_err = result_or_err.unwrap();

    if module_or_err.is_err() {
        #[cfg(debug_assertions)]
        println!("Failed to convert to wasmi module.");
        return 64 * test_cases.len() as u32;
    }
    let module = module_or_err.unwrap();
    let instance_or_err = wasmi::ModuleInstance::new(&module, &wasmi::ImportsBuilder::default());
    if instance_or_err.is_err() {
        #[cfg(debug_assertions)]
        println!("Failed to convert to wasmi instance.");
        return 64 * test_cases.len() as u32;
    }
    let instance = instance_or_err.unwrap().assert_no_start();
    let candidate_func = wasmi_utils::func_by_name(&instance, "candidate").unwrap();

    let mut dist = 0;
    for (input, expected_output) in test_cases {
        let actual_output =
            wasmi::FuncInstance::invoke(&candidate_func, input, &mut wasmi::NopExternals);
        dist += hamming_distance(expected_output, &actual_output);
    }

    dist
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::wasmi_utils;
    use wasmi::{ImportsBuilder, ModuleInstance, TrapKind};

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

        let inputs: Vec<Input> = vec![vec![RuntimeValue::I32(2)], vec![RuntimeValue::I32(0)]];

        let expected_output: Vec<Output> = vec![
            Result::Ok(Some(RuntimeValue::I32(2))),
            Result::Err(wasmi::Trap::new(wasmi::TrapKind::DivisionByZero)),
        ];

        let div_func = wasmi_utils::func_by_name(&instance, "div").unwrap();
        let actual_outputs = invoke_with_inputs(&div_func, &inputs);

        assert_eq!(inputs.len(), actual_outputs.len());
        for (i, actual_output) in actual_outputs.iter().enumerate() {
            let expected_output = &expected_output[i];

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

    #[test]
    fn hamming_distance_test() {
        assert_eq!(
            1,
            hamming_distance(
                &Output::Ok(Some(RuntimeValue::I32(1))),
                &Output::Ok(Some(RuntimeValue::I32(0)))
            )
        );
        assert_eq!(
            3,
            hamming_distance(
                &Output::Ok(Some(RuntimeValue::I64(5))),
                &Output::Ok(Some(RuntimeValue::I64(2)))
            )
        );
        assert_eq!(
            4,
            hamming_distance(
                &Output::Ok(Some(RuntimeValue::F32(
                    nan_preserving_float::F32::from_bits(0xF0)
                ))),
                &Output::Ok(Some(RuntimeValue::F32(
                    nan_preserving_float::F32::from_bits(0xAA)
                )))
            )
        );
        assert_eq!(
            32,
            hamming_distance(
                &Output::Ok(Some(RuntimeValue::I32(3))),
                &Output::Err(Trap::new(TrapKind::DivisionByZero))
            )
        );
        assert_eq!(
            64,
            hamming_distance(
                &Output::Err(Trap::new(TrapKind::DivisionByZero)),
                &Output::Ok(Some(RuntimeValue::I64(3))),
            )
        );
    }

    #[test]
    #[should_panic(expected = "Doens't support void functions.")]
    fn hamming_distance_void_func_test() {
        hamming_distance(&Output::Ok(None), &Output::Ok(None));
    }
}