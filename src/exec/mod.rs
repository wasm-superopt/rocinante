pub mod wasmer;
pub mod wasmi;

const NUM_TEST_CASES: usize = 16;

pub enum InterpreterKind {
    Wasmi,
    Wasmer,
    Wasmtime,
}

pub trait Interpreter {
    fn new() -> Self;
    fn kind() -> InterpreterKind;
    fn print_test_cases();
    fn eval_test_cases(binary: &[u8]) -> u32;
}
