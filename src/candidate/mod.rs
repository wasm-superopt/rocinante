use crate::stoke::whitelist;
use parity_wasm::elements::Instruction;
use std::cmp::Ordering;
use std::result::Result;

#[derive(Eq, Debug, Clone)]
pub struct Candidate {
    instrs: Vec<Instruction>,

    // Post Condition
    return_type_len: i32,

    // Enumerative Search Specific Fields.
    next_index: usize,
    num_values_on_stack: i32,
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub enum AppendError {
    NextIndexOutOfBounds,
    StackUnderflow,
    StackOverflow,
}

impl Candidate {
    /// New WASM program with given length.
    pub fn new(length: usize) -> Self {
        Self {
            instrs: vec![Instruction::Nop; length],
            // TODO(taegyunkim): Support multiple return values.
            return_type_len: 1,
            next_index: 0,
            num_values_on_stack: 0,
        }
    }

    /// Attempts to append the instruction to current program and returns a new candidate.
    pub fn try_append(&self, instr: Instruction) -> Result<Self, AppendError> {
        if self.next_index >= self.instrs.len() {
            return Err(AppendError::NextIndexOutOfBounds);
        }

        let (pop_cnts, push_cnts) = whitelist::stack_cnt(&instr);
        if self.num_values_on_stack - pop_cnts < 0 {
            return Err(AppendError::StackUnderflow);
        }

        let num_instrs_left = (self.instrs.len() - self.next_index - 1) as i32;
        if self.return_type_len < self.num_values_on_stack - pop_cnts + push_cnts - num_instrs_left
        {
            return Err(AppendError::StackOverflow);
        }

        let mut candidate = self.clone();

        candidate.instrs[candidate.next_index] = instr;
        candidate.num_values_on_stack -= pop_cnts;
        candidate.num_values_on_stack += push_cnts;
        candidate.next_index += 1;

        Ok(candidate)
    }

    pub fn instrs(&self) -> &[Instruction] {
        &self.instrs
    }

    pub fn return_type_len(&self) -> i32 {
        self.return_type_len
    }

    pub fn next_index(&self) -> usize {
        self.next_index
    }

    pub fn num_values_on_stack(&self) -> i32 {
        self.num_values_on_stack
    }
}

impl Ord for Candidate {
    fn cmp(&self, other: &Self) -> Ordering {
        self.next_index.cmp(&other.next_index()).reverse()
    }
}

impl PartialOrd for Candidate {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl PartialEq for Candidate {
    fn eq(&self, other: &Self) -> bool {
        self.next_index == other.next_index()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn constructor_test() {
        let candidate = Candidate::new(5);

        assert_eq!(candidate.num_values_on_stack(), 0);
        assert_eq!(candidate.next_index(), 0);
        assert_eq!(candidate.return_type_len(), 1);
    }

    #[test]
    fn try_append_index_out_of_bounds_test() {
        let candidate: Candidate = Candidate::new(0);
        let result = candidate.try_append(Instruction::Nop);
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), AppendError::NextIndexOutOfBounds);
    }

    #[test]
    fn try_append_stack_underflow_test() {
        let candidate: Candidate = Candidate::new(1);
        assert_eq!(candidate.return_type_len(), 1);
        let result = candidate.try_append(Instruction::I32Add);
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), AppendError::StackUnderflow);
    }

    #[test]
    fn try_append_stack_overflow_test() {
        let mut candidate: Candidate = Candidate::new(3);
        let mut result = candidate.try_append(Instruction::I32Const(1));
        assert!(result.is_ok());
        candidate = result.unwrap();
        result = candidate.try_append(Instruction::I32Const(1));
        assert!(result.is_ok());
        candidate = result.unwrap();

        result = candidate.try_append(Instruction::I32Const(1));
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), AppendError::StackOverflow);
    }
}
