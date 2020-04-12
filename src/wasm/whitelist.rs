use parity_wasm::elements::Instruction;
use rand::seq::SliceRandom;
use rand::Rng;

const I32BINOP: [Instruction; 15] = [
    Instruction::I32Add,
    Instruction::I32Sub,
    Instruction::I32Mul,
    Instruction::I32DivS,
    Instruction::I32DivU,
    Instruction::I32RemS,
    Instruction::I32RemU,
    Instruction::I32And,
    Instruction::I32Or,
    Instruction::I32Xor,
    Instruction::I32Shl,
    Instruction::I32ShrS,
    Instruction::I32ShrU,
    Instruction::I32Rotl,
    Instruction::I32Rotr,
];

const I32UNOP: [Instruction; 3] = [
    Instruction::I32Clz,
    Instruction::I32Ctz,
    Instruction::I32Popcnt,
];

const I32RELOP: [Instruction; 10] = [
    Instruction::I32Eq,
    Instruction::I32Ne,
    Instruction::I32LtS,
    Instruction::I32LtU,
    Instruction::I32GtS,
    Instruction::I32GtU,
    Instruction::I32LeS,
    Instruction::I32LeU,
    Instruction::I32GeS,
    Instruction::I32GeU,
];

const I32TESTOP: [Instruction; 1] = [Instruction::I32Eqz];

const LOCALOP: [fn(u32) -> Instruction; 3] = [
    |i| Instruction::GetLocal(i),
    |i| Instruction::SetLocal(i),
    |i| Instruction::TeeLocal(i),
];

pub struct Whitelist {
    _num_params: usize,
    // TODO(taegyunkim): Support increasing the number of locals.
    _num_locals: usize,
    // TODO(taegyunkim): Support other primitive types.
    _constants: Vec<i32>,

    instrs: Vec<Instruction>,
}

impl Whitelist {
    pub fn new(num_params: usize, num_locals: usize, constants: &[i32]) -> Self {
        let mut instrs = Vec::new();
        instrs.extend_from_slice(&I32BINOP);
        instrs.extend_from_slice(&I32UNOP);
        instrs.extend_from_slice(&I32RELOP);
        instrs.extend_from_slice(&I32TESTOP);

        for idx in 0..(num_params + num_locals) as u32 {
            instrs.push(Instruction::GetLocal(idx));
            instrs.push(Instruction::SetLocal(idx));
            instrs.push(Instruction::TeeLocal(idx));
        }

        for c in constants {
            instrs.push(Instruction::I32Const(*c));
        }

        instrs.shuffle(&mut rand::thread_rng());

        Self {
            _num_params: num_params,
            _num_locals: num_locals,
            _constants: constants.to_vec(),
            instrs,
        }
    }

    /// Returns one single whitelisted instruction.
    pub fn sample<R: Rng + ?Sized>(&self, rng: &mut R) -> Instruction {
        self.instrs.choose(rng).unwrap().clone()
    }

    pub fn sample_i32_const<R: Rng + ?Sized>(&self, rng: &mut R) -> i32 {
        *self._constants.choose(rng).unwrap()
    }

    /// Checks whether the given instruction is whitelisted or not.
    pub fn is_instr_whitelisted(&self, instr: &Instruction) -> bool {
        // NOTE(taegyunkim)
        *instr == Instruction::Nop || self.instrs.contains(&instr)
    }

    /// Returns a pair of numbers, the number of values the given instruction pops from the WASM
    /// runtime stack, and the number of values the given instruction pushes to the stack.
    pub fn push_pop_cnts(&self, instr: &Instruction) -> (i32, i32) {
        match *instr {
            // i32 binary operators
            Instruction::I32Add
            | Instruction::I32Sub
            | Instruction::I32Mul
            | Instruction::I32DivS
            | Instruction::I32DivU
            | Instruction::I32RemS
            | Instruction::I32RemU
            | Instruction::I32And
            | Instruction::I32Or
            | Instruction::I32Xor
            | Instruction::I32Shl
            | Instruction::I32ShrS
            | Instruction::I32ShrU
            | Instruction::I32Rotl
            | Instruction::I32Rotr => (2, 1),
            // i32 relative operators
            Instruction::I32Eq
            | Instruction::I32Ne
            | Instruction::I32LtS
            | Instruction::I32LtU
            | Instruction::I32GtS
            | Instruction::I32GtU
            | Instruction::I32LeS
            | Instruction::I32LeU
            | Instruction::I32GeS
            | Instruction::I32GeU => (2, 1),
            // i32 testop
            Instruction::I32Eqz => (1, 1),
            // i32 unop
            Instruction::I32Clz | Instruction::I32Ctz | Instruction::I32Popcnt => (1, 1),
            Instruction::I32Const(_) => (0, 1),
            Instruction::GetLocal(_) => (0, 1),
            Instruction::SetLocal(_) => (1, 0),
            Instruction::TeeLocal(_) => (1, 1),
            Instruction::Nop => (0, 0),
            _ => {
                if self.instrs.contains(instr) {
                    panic!("Forgot to implement instruction {}", instr);
                } else {
                    panic!("Instruction {} not supported.", instr);
                }
            }
        }
    }

    /// Returns an instruction that is in the same equivalence class.
    pub fn get_equiv_instr<R: Rng + ?Sized>(
        &self,
        rng: &mut R,
        instr: &Instruction,
    ) -> Instruction {
        match *instr {
            Instruction::I32Add
            | Instruction::I32Sub
            | Instruction::I32Mul
            | Instruction::I32DivS
            | Instruction::I32DivU
            | Instruction::I32RemS
            | Instruction::I32RemU
            | Instruction::I32And
            | Instruction::I32Or
            | Instruction::I32Xor
            | Instruction::I32Shl
            | Instruction::I32ShrS
            | Instruction::I32ShrU
            | Instruction::I32Rotl
            | Instruction::I32Rotr => I32BINOP.choose(rng).unwrap().clone(),
            Instruction::I32Eq
            | Instruction::I32Ne
            | Instruction::I32LtS
            | Instruction::I32LtU
            | Instruction::I32GtS
            | Instruction::I32GtU
            | Instruction::I32LeS
            | Instruction::I32LeU
            | Instruction::I32GeS
            | Instruction::I32GeU => I32RELOP.choose(rng).unwrap().clone(),
            Instruction::I32Eqz => Instruction::I32Eqz,
            Instruction::I32Clz | Instruction::I32Ctz | Instruction::I32Popcnt => {
                I32UNOP.choose(rng).unwrap().clone()
            }
            Instruction::I32Const(i) => Instruction::I32Const(i),
            Instruction::GetLocal(i) | Instruction::SetLocal(i) | Instruction::TeeLocal(i) => {
                LOCALOP.choose(rng).unwrap()(i)
            }
            Instruction::Nop => Instruction::Nop,
            _ => {
                if self.instrs.contains(instr) {
                    panic!("Forgot to implement instruction {}", instr);
                } else {
                    panic!("Instruction {} not supported.", instr);
                }
            }
        }
    }

    pub fn iter(&self) -> std::slice::Iter<Instruction> {
        self.instrs.iter()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn check_test() {
        let whitelist = Whitelist::new(3, 0, &[0, 1, 2]);

        for instr in &I32BINOP {
            assert!(whitelist.is_instr_whitelisted(instr));
        }
        for instr in &I32UNOP {
            assert!(whitelist.is_instr_whitelisted(instr));
        }
        for instr in &I32RELOP {
            assert!(whitelist.is_instr_whitelisted(instr));
        }
        for instr in &LOCALOP {
            assert!(whitelist.is_instr_whitelisted(&instr(0)));
        }

        for instr in &[
            Instruction::I32Eqz,
            Instruction::GetLocal(2),
            Instruction::Nop,
        ] {
            assert!(&whitelist.is_instr_whitelisted(instr));
        }
    }

    #[test]
    fn stack_cnt_whitelist_test() {
        let whitelist = Whitelist::new(1, 0, &[0, 1, 2]);
        for instr in whitelist.instrs.iter() {
            let _cnts = whitelist.push_pop_cnts(instr);
        }
    }
}
