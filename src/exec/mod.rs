pub mod wasmer;
pub mod wasmtime;

const NUM_TEST_CASES: usize = 16;

/// When computing the cost of candidate WASM binaries, this value is add to the total number of
/// return type bits to differentiate invalid WASMs from valid WASMs returning all outputs
/// incorrectly.
const EPSILON: u32 = 1;

#[derive(Debug, Clone, Copy, PartialEq, Eq, EnumString)]
pub enum InterpreterKind {
    Wasmer,
    Wasmtime,
}

pub trait Interpreter {
    fn kind(&self) -> InterpreterKind;
    fn print_test_cases(&self);

    // NOTE(taegyunkim): The return type of this function is unsigned instead of
    // signed because it represents the sum of hamming distances. When it overflows,
    // rust will panic.
    fn eval_test_cases(&self, binary: &[u8]) -> u32;

    /// Score for an invalid WASM program.
    fn score_invalid(&self) -> u32 {
        self.num_test_cases() as u32 * (self.return_bit_width() + EPSILON)
    }

    fn add_test_case(&mut self, input: &[::wasmi::RuntimeValue]);

    fn return_type_len(&self) -> usize;

    fn return_bit_width(&self) -> u32;

    fn num_test_cases(&self) -> usize;
}

pub fn get_interpreter(
    kind: InterpreterKind,
    spec: &[u8],
    func_name: &str,
) -> Box<dyn Interpreter> {
    match kind {
        InterpreterKind::Wasmer => Box::new(wasmer::Wasmer::new(spec, func_name)),
        InterpreterKind::Wasmtime => Box::new(wasmtime::Wasmtime::new(spec, func_name)),
    }
}
