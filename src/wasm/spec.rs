use crate::parity_wasm_utils;
use parity_wasm::elements::serialize;
use parity_wasm::elements::{FuncBody, FunctionType, Instruction, Instructions, ValueType};

/// Struct to hold spec function metadata.
#[derive(Debug, Clone, PartialEq)]
pub struct Spec {
    // Fields representing the spec.
    spec_func_type: FunctionType,
    spec_local_types: Vec<ValueType>,
    spec_func_body: FuncBody,

    /// This field contains WASM binary generated from above func_type, with function name
    /// 'candidate'. It is initialized once when this struct is initialized and reused to avoid
    /// multiple conversions to binary format which are costly.
    binary: Vec<u8>,
    binary_len: usize,
}

impl Spec {
    pub fn new(spec_func_type: &FunctionType, spec_func_body: &FuncBody) -> Self {
        let mut binary = parity_wasm_utils::build_module(
            "candidate",
            &spec_func_type,
            FuncBody::new(vec![], Instructions::new(vec![])),
        )
        .to_bytes()
        .unwrap();

        let locals = spec_func_body.locals();
        let len = spec_func_body.code().elements().len();

        // The last four bytes represent the code section of WASM, and always have 3, 1, 1, 0,
        // The value 3 represents the length of the sequence of bytes that follows, and the first
        // 1 means that there is one function. The last two bytes actually represent the function,
        // which can be translated to Nop, and Unreachable.
        binary = binary[0..binary.len() - 4].to_vec();
        // Reserve space ahead to avoid expensive memory operations during search.
        binary.reserve(2 + len);
        let binary_len = binary.len();

        // Keep track of the local types of the spec.
        let mut spec_local_types = Vec::new();
        for l in locals {
            for _ in 0..l.count() {
                spec_local_types.push(l.value_type());
            }
        }

        Self {
            spec_func_type: spec_func_type.clone(),
            spec_local_types,
            spec_func_body: spec_func_body.clone(),
            binary,
            binary_len,
        }
    }

    pub fn get_spec_func_body(&self) -> &FuncBody {
        &self.spec_func_body
    }

    pub fn spec_func_type(&self) -> &FunctionType {
        &self.spec_func_type
    }

    // TODO(taegyunkim): Support multiple value return type.
    pub fn return_type_len(&self) -> usize {
        match self.spec_func_type.return_type() {
            Some(_) => 1,
            None => 0,
        }
    }

    pub fn spec_param_types(&self) -> &[ValueType] {
        &self.spec_func_type.params()
    }

    pub fn spec_local_types(&self) -> &[ValueType] {
        &self.spec_local_types
    }

    /// Returns the number of parameterss. This doesn't include the number of locals.
    ///
    /// See [num_locals](Spec.num_locals)
    pub fn num_params(&self) -> usize {
        self.spec_func_type.params().len()
    }

    /// Returns the number of locals. This doesn't include the number of parameters.
    ///
    /// See [num_params](Spec.num_params).
    pub fn num_locals(&self) -> usize {
        self.spec_local_types.len()
    }

    /// Returns the list of types for all parameters and local variables. The length of returned
    /// slice equals to `[num_params](Spec.num_params) + [num_locals](Spec.num_locals)`
    pub fn get_param_and_local_types(&self) -> &[ValueType] {
        &self.spec_local_types
    }

    /// Returns the number of instructions, excluding END instruction at the end.
    ///
    /// We assume that the spec function has only one END instruction at the end of the function,
    /// and doesn't contain any blocks, constrol flows within it, which would also use END.
    pub fn num_instrs(&self) -> usize {
        self.spec_func_body.code().elements().len() - 1
    }

    pub fn get_binary_with_instrs(&mut self, instrs: &[Instruction]) -> &[u8] {
        // NOTE(taegyunkim): As commented in the constructor we need to append an END instruction,
        // to make this a valid function representation.
        let mut instrs = instrs.to_vec();
        instrs.push(Instruction::End);

        let func_body = FuncBody::new(
            self.spec_func_body.locals().to_vec(),
            Instructions::new(instrs),
        );

        // TODO(taegyunkim): Avoid conversion to FuncBody and convert instruction list to binary.
        let func_binary = serialize::<FuncBody>(func_body).unwrap();

        self.binary.truncate(self.binary_len);
        self.binary.extend(&[func_binary.len() as u8 + 1, 1]);
        self.binary.extend(func_binary);

        &self.binary
    }
}
