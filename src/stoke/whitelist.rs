use parity_wasm::elements::{Instruction, ValueType};
use rand::seq::SliceRandom;
use rand::Rng;

// TODO(taegyunkim): Figure out a way to check all cases are covered whenever
// this is used.
#[derive(Debug, PartialEq, Eq, Copy, Clone)]
pub enum WhitelistedInstruction {
    I32Add,
    I32Sub,
    I32Mul,
    I32DivS,
    I32DivU,
    I32RemS,
    I32RemU,
    I32And,
    I32Or,
    I32Xor,
    I32Shl,
    I32ShrS,
    I32ShrU,
    I32Rotl,
    I32Rotr,
    I32Const(i32),
    GetLocal(u32),
    SetLocal(u32),
    TeeLocal(u32),
    End,
    Nop,
}

impl WhitelistedInstruction {
    pub fn sample<R: Rng + ?Sized>(
        rng: &mut R,
        param_types: &[ValueType],
        // TODO: Support increasing the number of locals.
        _local_types: &mut Vec<ValueType>,
    ) -> WhitelistedInstruction {
        match rng.gen_range(0, 21) {
            0 => WhitelistedInstruction::I32Add,
            1 => WhitelistedInstruction::I32Sub,
            2 => WhitelistedInstruction::I32Mul,
            3 => WhitelistedInstruction::I32DivS,
            4 => WhitelistedInstruction::I32DivU,
            5 => WhitelistedInstruction::I32RemS,
            6 => WhitelistedInstruction::I32RemU,
            7 => WhitelistedInstruction::I32And,
            8 => WhitelistedInstruction::I32Or,
            9 => WhitelistedInstruction::I32Xor,
            10 => WhitelistedInstruction::I32Shl,
            11 => WhitelistedInstruction::I32ShrS,
            12 => WhitelistedInstruction::I32ShrU,
            13 => WhitelistedInstruction::I32Rotl,
            14 => WhitelistedInstruction::I32Rotr,
            15 => {
                let operands = vec![-2, -1, 0, 1, 2];
                WhitelistedInstruction::I32Const(*operands.choose(rng).unwrap())
            }
            16 => {
                let idx = rng.gen_range(0, param_types.len()) as u32;
                WhitelistedInstruction::GetLocal(idx)
            }
            17 => {
                let idx = rng.gen_range(0, param_types.len()) as u32;
                WhitelistedInstruction::SetLocal(idx)
            }
            18 => {
                let idx = rng.gen_range(0, param_types.len()) as u32;
                WhitelistedInstruction::TeeLocal(idx)
            }
            19 => WhitelistedInstruction::End,
            _ => WhitelistedInstruction::Nop,
        }
    }
}

impl std::fmt::Display for WhitelistedInstruction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match *self {
            WhitelistedInstruction::I32Add => write!(f, "i32.add"),
            WhitelistedInstruction::I32Sub => write!(f, "i32.sub"),
            WhitelistedInstruction::I32Mul => write!(f, "i32.mul"),
            WhitelistedInstruction::I32DivS => write!(f, "i32.divs"),
            WhitelistedInstruction::I32DivU => write!(f, "i32.divu"),
            WhitelistedInstruction::I32RemS => write!(f, "i32.rems"),
            WhitelistedInstruction::I32RemU => write!(f, "i32.remu"),
            WhitelistedInstruction::I32And => write!(f, "i32.and"),
            WhitelistedInstruction::I32Or => write!(f, "i32.or"),
            WhitelistedInstruction::I32Xor => write!(f, "i32.xor"),
            WhitelistedInstruction::I32Shl => write!(f, "i32.shl"),
            WhitelistedInstruction::I32ShrS => write!(f, "i32.shrs"),
            WhitelistedInstruction::I32ShrU => write!(f, "i32.shru"),
            WhitelistedInstruction::I32Rotl => write!(f, "i32.rotl"),
            WhitelistedInstruction::I32Rotr => write!(f, "i32.rotr"),
            WhitelistedInstruction::I32Const(i) => write!(f, "i32.const {}", i),
            WhitelistedInstruction::GetLocal(i) => write!(f, "get_local {}", i),
            WhitelistedInstruction::SetLocal(i) => write!(f, "set_local {}", i),
            WhitelistedInstruction::TeeLocal(i) => write!(f, "tee_local {}", i),
            WhitelistedInstruction::End => write!(f, "end"),
            WhitelistedInstruction::Nop => write!(f, "nop"),
        }
    }
}

impl From<Instruction> for WhitelistedInstruction {
    fn from(instr: Instruction) -> Self {
        match instr {
            Instruction::I32Add => WhitelistedInstruction::I32Add,
            Instruction::I32Sub => WhitelistedInstruction::I32Sub,
            Instruction::I32Mul => WhitelistedInstruction::I32Mul,
            Instruction::I32DivS => WhitelistedInstruction::I32DivS,
            Instruction::I32DivU => WhitelistedInstruction::I32DivU,
            Instruction::I32RemS => WhitelistedInstruction::I32RemS,
            Instruction::I32RemU => WhitelistedInstruction::I32RemU,
            Instruction::I32And => WhitelistedInstruction::I32And,
            Instruction::I32Or => WhitelistedInstruction::I32Or,
            Instruction::I32Xor => WhitelistedInstruction::I32Xor,
            Instruction::I32Shl => WhitelistedInstruction::I32Shl,
            Instruction::I32ShrS => WhitelistedInstruction::I32ShrS,
            Instruction::I32ShrU => WhitelistedInstruction::I32ShrU,
            Instruction::I32Rotl => WhitelistedInstruction::I32Rotl,
            Instruction::I32Rotr => WhitelistedInstruction::I32Rotr,
            Instruction::I32Const(i) => WhitelistedInstruction::I32Const(i),
            Instruction::GetLocal(i) => WhitelistedInstruction::GetLocal(i),
            Instruction::SetLocal(i) => WhitelistedInstruction::SetLocal(i),
            Instruction::TeeLocal(i) => WhitelistedInstruction::TeeLocal(i),
            Instruction::End => WhitelistedInstruction::End,
            Instruction::Nop => WhitelistedInstruction::Nop,
            _ => panic!("{} not implemented", instr),
        }
    }
}

impl Into<Instruction> for WhitelistedInstruction {
    fn into(self) -> Instruction {
        match self {
            WhitelistedInstruction::I32Add => Instruction::I32Add,
            WhitelistedInstruction::I32Sub => Instruction::I32Sub,
            WhitelistedInstruction::I32Mul => Instruction::I32Mul,
            WhitelistedInstruction::I32DivS => Instruction::I32DivS,
            WhitelistedInstruction::I32DivU => Instruction::I32DivU,
            WhitelistedInstruction::I32RemS => Instruction::I32RemS,
            WhitelistedInstruction::I32RemU => Instruction::I32RemU,
            WhitelistedInstruction::I32And => Instruction::I32And,
            WhitelistedInstruction::I32Or => Instruction::I32Or,
            WhitelistedInstruction::I32Xor => Instruction::I32Xor,
            WhitelistedInstruction::I32Shl => Instruction::I32Shl,
            WhitelistedInstruction::I32ShrS => Instruction::I32ShrS,
            WhitelistedInstruction::I32ShrU => Instruction::I32ShrU,
            WhitelistedInstruction::I32Rotl => Instruction::I32Rotl,
            WhitelistedInstruction::I32Rotr => Instruction::I32Rotr,
            WhitelistedInstruction::I32Const(i) => Instruction::I32Const(i),
            WhitelistedInstruction::GetLocal(i) => Instruction::GetLocal(i),
            WhitelistedInstruction::SetLocal(i) => Instruction::SetLocal(i),
            WhitelistedInstruction::TeeLocal(i) => Instruction::TeeLocal(i),
            WhitelistedInstruction::End => Instruction::End,
            WhitelistedInstruction::Nop => Instruction::Nop,
        }
    }
}
