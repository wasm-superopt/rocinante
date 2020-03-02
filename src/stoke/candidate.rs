use crate::parity_wasm_utils;
use crate::stoke::whitelist;
use parity_wasm::elements::serialize;
use parity_wasm::elements::{
    FuncBody, FunctionType, Instruction, Instructions, Local, Module, ValueType,
};
use rand::seq::SliceRandom;
use rand::Rng;

#[derive(Debug, Clone, PartialEq)]
pub struct Candidate {
    // Fields representing the spec.
    spec_func_type: FunctionType,
    spec_local_types: Vec<ValueType>,
    spec_func_body: FuncBody,

    /// This field contains WASM binary generated from above func_type, with function name
    /// 'candidate'. It is initialized once when this struct is initialized and reused to avoid
    /// multiple conversions to binary format which are costly.
    binary: Vec<u8>,
    binary_len: usize,

    // Below are fields representing current candidate.
    instrs: Vec<Instruction>,
    /// The list of constants to use for synthesis.
    constants: Vec<i32>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum StackState {
    Valid,
    Invalid(i32),
}

impl Candidate {
    pub fn new(
        spec_func_type: &FunctionType,
        spec_func_body: &FuncBody,
        constants: Vec<i32>,
    ) -> Self {
        whitelist::check_instrs(spec_func_body.code().elements());

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
            // NOTE(taegyunkim): The original spec instruction list has an END instruction at the
            // end, so subtract one for that. We assume that we want to synthesize a function that
            // simply returns a value without any control flow.
            instrs: vec![Instruction::Nop; len - 1],
            constants,
        }
    }

    pub fn get_rand_instr<R: Rng + ?Sized>(&self, rng: &mut R) -> (usize, Instruction) {
        let indices = rand::seq::index::sample(rng, self.instrs.len(), 1);
        (indices.index(0), self.instrs[indices.index(0)].clone())
    }

    pub fn get_equiv_local_idx<R: Rng + ?Sized>(&self, rng: &mut R, i: u32) -> u32 {
        let i = i as usize;
        let typ: &ValueType = if i < self.spec_func_type.params().len() {
            &self.spec_func_type.params()[i]
        } else if i < self.spec_func_type.params().len() + self.spec_local_types.len() {
            &self.spec_local_types[i - self.spec_func_type.params().len()]
        } else {
            panic!("local index out of bounds: {}", i);
        };

        let mut equiv_indices = Vec::new();
        for (i, param_type) in self.spec_func_type.params().iter().enumerate() {
            if param_type == typ {
                equiv_indices.push(i);
            }
        }

        for (i, local_type) in self.spec_local_types.iter().enumerate() {
            if local_type == typ {
                equiv_indices.push(i + self.spec_func_type.params().len());
            }
        }

        assert!(!equiv_indices.is_empty());

        *equiv_indices.choose(rng).unwrap() as u32
    }

    pub fn sample_local_idx<R: Rng + ?Sized>(&self, rng: &mut R) -> u32 {
        rng.gen_range(
            0,
            self.spec_func_type.params().len() + self.spec_local_types.len(),
        ) as u32
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

    pub fn get_spec_func_body(&self) -> &FuncBody {
        &self.spec_func_body
    }

    pub fn get_func_body(&self) -> FuncBody {
        // TODO(taegyunkim): For candidate programs that don't use locals, this can be removed.
        let locals: Vec<Local> = self
            .spec_local_types
            .iter()
            .map(|typ| Local::new(1, *typ))
            .collect();

        // NOTE(taegyunkim): As commented in the constructor we need to append an END instruction,
        // to make this a valid function representation.
        let mut instrs = self.instrs.clone();
        instrs.push(Instruction::End);

        FuncBody::new(locals, Instructions::new(instrs))
    }

    pub fn spec_func_type(&self) -> &FunctionType {
        &self.spec_func_type
    }

    pub fn to_module(&self) -> Module {
        parity_wasm_utils::build_module("candidate", &self.spec_func_type, self.get_func_body())
    }

    pub fn get_binary(&mut self) -> &[u8] {
        // TODO(taegyunkim): Avoid conversion to FuncBody and convert instruction list to binary.
        let func_binary = serialize::<FuncBody>(self.get_func_body()).unwrap();

        self.binary.truncate(self.binary_len);
        self.binary.extend(&[func_binary.len() as u8 + 1, 1]);
        self.binary.extend(func_binary);

        // TODO(taegyunkim): Add a test.
        assert_eq!(self.binary, self.to_module().to_bytes().unwrap());
        &self.binary
    }

    pub fn check_stack(&self) -> StackState {
        check_instrs(self.instrs())
    }
}

pub fn check_instrs(instrs: &[Instruction]) -> StackState {
    let mut cnt: i32 = 0;
    let mut valid = true;
    for instr in instrs {
        let (pop, push) = whitelist::stack_cnt(instr);
        cnt -= pop;
        if cnt < 0 {
            valid = false;
        }
        cnt += push;
    }
    if cnt == 1 && valid {
        StackState::Valid
    } else {
        StackState::Invalid(cnt)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn check_instrs_test() {
        assert_eq!(
            StackState::Invalid(-2),
            check_instrs(&vec![
                Instruction::I32Const(-1),
                Instruction::TeeLocal(0),
                Instruction::I32GeS,
                Instruction::I32ShrU,
                Instruction::I32And,
            ])
        );

        assert_eq!(
            StackState::Invalid(-1),
            check_instrs(&vec![
                Instruction::GetLocal(0),
                Instruction::I32Ctz,
                Instruction::TeeLocal(0),
                Instruction::I32LtS,
                Instruction::I32LeS,
            ])
        );

        assert_eq!(
            StackState::Invalid(0),
            check_instrs(&vec![
                Instruction::GetLocal(0),  // 1
                Instruction::I32Const(-2), // 2
                Instruction::I32GtS,       // 1
                Instruction::TeeLocal(0),  // 1
                Instruction::SetLocal(0),  // 0
            ])
        );
    }
}
