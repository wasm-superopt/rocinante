use crate::wasm::Whitelist;
use parity_wasm::elements::Instruction;
use rand::Rng;
use std::cmp::Ordering;
use std::result::Result;

#[derive(Clone, Debug, Eq)]
pub struct Candidate {
    instrs: Vec<Instruction>,

    // Enumerative Search Specific Fields.
    next_index: usize,
    num_values_on_stack: i32,
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

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum AppendError {
    NextIndexOutOfBounds,
    StackUnderflow,
    StackOverflow,
}

#[derive(Debug, Clone, PartialEq)]
pub enum StackState {
    Valid,
    Invalid(i32),
}

impl Candidate {
    /// New WASM program with given length.
    pub fn new(max_length: usize) -> Self {
        Self {
            instrs: vec![Instruction::Nop; max_length],
            next_index: 0,
            num_values_on_stack: 0,
        }
    }

    pub fn from_instrs(instrs: Vec<Instruction>) -> Self {
        // TODO(taegyunkim): Properly update num_values_on_stack.
        Self {
            instrs,
            next_index: 0,
            num_values_on_stack: 0,
        }
    }

    /// Attempts to append the instruction to current program and returns a new candidate.
    pub fn try_append(
        &self,
        instr_whitelist: &Whitelist,
        instr: Instruction,
    ) -> Result<Self, AppendError> {
        if self.next_index >= self.instrs.len() {
            return Err(AppendError::NextIndexOutOfBounds);
        }

        let (pop_cnts, push_cnts) = instr_whitelist.push_pop_cnts(&instr);
        if self.num_values_on_stack - pop_cnts < 0 {
            return Err(AppendError::StackUnderflow);
        }

        let num_instrs_left = (self.instrs.len() - self.next_index - 1) as i32;
        // TODO(taegyunkim): Support multiple return values.
        let return_type_len = 1;
        if return_type_len < self.num_values_on_stack - pop_cnts + push_cnts - num_instrs_left {
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

    pub fn instrs_mut(&mut self) -> &mut Vec<Instruction> {
        &mut self.instrs
    }

    pub fn next_index(&self) -> usize {
        self.next_index
    }

    pub fn num_values_on_stack(&self) -> i32 {
        self.num_values_on_stack
    }

    /// Removes Nop instructions.
    pub fn strip_nops(&mut self) {
        self.instrs = self
            .instrs
            .iter()
            .cloned()
            .filter(|instr| *instr != Instruction::Nop)
            .collect();
    }

    pub fn get_rand_instr<R: Rng + ?Sized>(&self, rng: &mut R) -> (usize, Instruction) {
        let indices = rand::seq::index::sample(rng, self.instrs.len(), 1);
        (indices.index(0), self.instrs[indices.index(0)].clone())
    }

    pub fn is_stack_valid(&self, instr_whitelist: &Whitelist) -> StackState {
        let mut cnt: i32 = 0;
        let mut valid = true;
        for instr in &self.instrs {
            let (pop, push) = instr_whitelist.push_pop_cnts(instr);
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
}
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn constructor_test() {
        let candidate = Candidate::new(5);

        assert_eq!(candidate.num_values_on_stack(), 0);
        assert_eq!(candidate.next_index(), 0);
    }

    #[test]
    fn try_append_index_out_of_bounds_test() {
        let instr_whitelist = Whitelist::new(1, 0, &[]);
        let candidate: Candidate = Candidate::new(0);
        let result = candidate.try_append(&instr_whitelist, Instruction::Nop);
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), AppendError::NextIndexOutOfBounds);
    }

    #[test]
    fn try_append_stack_underflow_test() {
        let instr_whitelist = Whitelist::new(1, 0, &[]);
        let candidate: Candidate = Candidate::new(1);

        let result = candidate.try_append(&instr_whitelist, Instruction::I32Add);
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), AppendError::StackUnderflow);
    }

    #[test]
    fn try_append_stack_overflow_test() {
        let instr_whitelist = Whitelist::new(1, 0, &[1]);
        let mut candidate: Candidate = Candidate::new(3);
        let mut result = candidate.try_append(&instr_whitelist, Instruction::I32Const(1));
        assert!(result.is_ok());
        candidate = result.unwrap();
        result = candidate.try_append(&instr_whitelist, Instruction::I32Const(1));
        assert!(result.is_ok());
        candidate = result.unwrap();

        result = candidate.try_append(&instr_whitelist, Instruction::I32Const(1));
        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), AppendError::StackOverflow);
    }
}
