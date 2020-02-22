use crate::parity_wasm_utils;
use parity_wasm::elements::serialize;
use parity_wasm::elements::{
    FuncBody, FunctionType, Instruction, Instructions, Local, Module, ValueType,
};
use rand::seq::SliceRandom;
use rand::Rng;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Candidate {
    func_type: FunctionType,
    local_types: Vec<ValueType>,
    instrs: Vec<Instruction>,
    stack_cnt: i32,
    constants: Vec<i32>,
    /// This field contains WASM binary generated from above func_type, with function name
    /// 'candidate'. It is initialized once when this struct is initialized and reused to avoid
    /// multiple conversions to binary format which are costly.
    binary: Vec<u8>,
    binary_len: usize,
}

impl Candidate {
    pub fn new(
        func_type: &FunctionType,
        locals: &[Local],
        len: usize,
        constants: Vec<i32>,
    ) -> Self {
        let mut binary = parity_wasm_utils::build_module(
            "candidate",
            &func_type,
            FuncBody::new(vec![], Instructions::new(vec![])),
        )
        .to_bytes()
        .unwrap();

        // The last four bytes represent the code section of WASM, and always have 3, 1, 1, 0,
        // The value 3 represents the length of the sequence of bytes that follows, and the first
        // 1 means that there is one function. The last two bytes actually represent the function,
        // which can be translated to Nop, and Unreachable.
        binary = binary[0..binary.len() - 4].to_vec();
        // Reserve space ahead to avoid expensive memory operations during search.
        binary.reserve(2 + len);
        let binary_len = binary.len();

        // Keep track of the local types of the spec.
        let mut local_types = Vec::new();
        for l in locals {
            for _ in 0..l.count() {
                local_types.push(l.value_type());
            }
        }

        Self {
            func_type: func_type.clone(),
            local_types,
            // NOTE(taegyunkim): The original spec instruction list has an END instruction at the
            // end, so subtract one for that. We assume that we want to synthesize a function that
            // simply returns a value without any control flow.
            instrs: vec![Instruction::Nop; len - 1],
            stack_cnt: 0,
            constants,
            binary,
            binary_len,
        }
    }

    pub fn inc_stack_cnt(&mut self, n: i32) {
        self.stack_cnt += n;
    }

    pub fn dec_stack_cnt(&mut self, n: i32) {
        self.stack_cnt -= n;
    }

    pub fn stack_cnt(&self) -> i32 {
        self.stack_cnt
    }

    pub fn get_rand_instr<R: Rng + ?Sized>(&self, rng: &mut R) -> (usize, Instruction) {
        let indices = rand::seq::index::sample(rng, self.instrs.len(), 1);
        (indices.index(0), self.instrs[indices.index(0)].clone())
    }

    pub fn get_equiv_local_idx<R: Rng + ?Sized>(&self, rng: &mut R, i: u32) -> u32 {
        let i = i as usize;
        let typ: &ValueType = if i < self.func_type.params().len() {
            &self.func_type.params()[i]
        } else if i < self.func_type.params().len() + self.local_types.len() {
            &self.local_types[i - self.func_type.params().len()]
        } else {
            panic!("local index out of bounds: {}", i);
        };

        let mut equiv_indices = Vec::new();
        for (i, param_type) in self.func_type.params().iter().enumerate() {
            if param_type == typ {
                equiv_indices.push(i);
            }
        }

        for (i, local_type) in self.local_types.iter().enumerate() {
            if local_type == typ {
                equiv_indices.push(i + self.func_type.params().len());
            }
        }

        assert!(!equiv_indices.is_empty());

        *equiv_indices.choose(rng).unwrap() as u32
    }

    pub fn sample_local_idx<R: Rng + ?Sized>(&self, rng: &mut R) -> u32 {
        rng.gen_range(0, self.func_type.params().len() + self.local_types.len()) as u32
    }

    pub fn sample_i32<R: Rng + ?Sized>(&self, rng: &mut R) -> i32 {
        *self.constants.choose(rng).unwrap()
    }

    pub fn instrs(&self) -> &[Instruction] {
        &self.instrs
    }

    pub fn instrs_mut(&mut self) -> &mut Vec<Instruction> {
        &mut self.instrs
    }

    pub fn to_func_body(&self) -> FuncBody {
        // TODO(taegyunkim): For candidate programs that don't use locals, this can be removed.
        let locals: Vec<Local> = self
            .local_types
            .iter()
            .map(|typ| Local::new(1, *typ))
            .collect();

        // NOTE(taegyunkim): As commented in the constructor we need to append an END instruction,
        // to make this a valid function representation.
        let mut instrs = self.instrs.clone();
        instrs.push(Instruction::End);

        FuncBody::new(locals, Instructions::new(instrs))
    }

    pub fn to_module(&self) -> Module {
        parity_wasm_utils::build_module("candidate", &self.func_type, self.to_func_body())
    }

    pub fn get_binary(&mut self) -> &[u8] {
        // TODO(taegyunkim): Avoid conversion to FuncBody and convert instruction list to binary.
        let func_binary = serialize::<FuncBody>(self.to_func_body()).unwrap();

        self.binary.truncate(self.binary_len);
        self.binary.extend(&[func_binary.len() as u8 + 1, 1]);
        self.binary.extend(func_binary);

        // TODO(taegyunkim): Add a test.
        //assert_eq!(self.binary, self.to_module().to_bytes().unwrap());
        &self.binary
    }
}
