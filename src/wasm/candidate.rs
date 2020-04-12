use crate::wasm::Whitelist;
use parity_wasm::elements::Instruction;
use rand::Rng;

#[derive(Debug, Clone)]
pub struct Candidate {
    instrs: Vec<Instruction>,
}

#[derive(Debug, Clone, Eq, PartialEq)]
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
        }
    }

    pub fn from_instrs(instrs: Vec<Instruction>) -> Self {
        // TODO(taegyunkim): Properly update num_values_on_stack.
        Self { instrs }
    }

    pub fn instrs(&self) -> &[Instruction] {
        &self.instrs
    }

    pub fn instrs_mut(&mut self) -> &mut Vec<Instruction> {
        &mut self.instrs
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
