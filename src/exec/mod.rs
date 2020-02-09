pub mod wasmer;
pub mod wasmi;
pub mod wasmtime;

const NUM_TEST_CASES: usize = 16;

/// When computing the cost of candidate WASM binaries, this value is add to the total number of
/// return type bits to differentiate invalid WASMs from valid WASMs returning all outputs
/// incorrectly.
const EPSILON: u32 = 1;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InterpreterKind {
    Wasmi,
    Wasmer,
    Wasmtime,
}

pub trait Interpreter {
    fn new() -> Self;
    fn kind(&self) -> InterpreterKind;
    fn generate_test_cases(&mut self, spec: &[u8], func_name: &str);
    fn print_test_cases(&self);

    // NOTE(taegyunkim): The return type of this function is unsigned instead of
    // signed because it represents the sum of hamming distances. When it overflows,
    // rust will panic.
    fn eval_test_cases(&self, candidate: &[u8]) -> u32;
}
