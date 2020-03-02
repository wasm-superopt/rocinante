use crate::stoke::Candidate;
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

const LOCALOP: [Instruction; 3] = [
    Instruction::GetLocal(0),
    Instruction::SetLocal(0),
    Instruction::TeeLocal(0),
];

const WHITELIST: [Instruction; 34] = [
    // i32 binop
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
    // i32 unop
    Instruction::I32Clz,
    Instruction::I32Ctz,
    Instruction::I32Popcnt,
    // i32 relop
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
    // i32 testop
    Instruction::I32Eqz,
    // i32 const
    Instruction::I32Const(0),
    // local op
    Instruction::GetLocal(0),
    Instruction::SetLocal(0),
    Instruction::TeeLocal(0),
    Instruction::Nop,
];

pub fn sample<R: Rng + ?Sized>(
    rng: &mut R,
    // TODO(taegyunkim): Support increasing the number of locals.
    candidate_func: &mut Candidate,
) -> Instruction {
    let instr = WHITELIST.choose(rng).unwrap().clone();
    match instr {
        Instruction::I32Const(_) => Instruction::I32Const(candidate_func.sample_i32(rng)),
        Instruction::GetLocal(_) => Instruction::GetLocal(candidate_func.sample_local_idx(rng)),
        Instruction::SetLocal(_) => Instruction::SetLocal(candidate_func.sample_local_idx(rng)),
        Instruction::TeeLocal(_) => Instruction::TeeLocal(candidate_func.sample_local_idx(rng)),
        _ => instr,
    }
}

pub fn check(instr: &Instruction) -> bool {
    match instr {
        Instruction::I32Const(_)
        | Instruction::GetLocal(_)
        | Instruction::SetLocal(_)
        | Instruction::TeeLocal(_)
        | Instruction::End => true,
        _ => WHITELIST.contains(instr),
    }
}

pub fn check_instrs(instrs: &[Instruction]) {
    for instr in instrs {
        if !check(instr) {
            panic!("{} not supported", instr);
        }
    }
}

pub fn stack_cnt(instr: &Instruction) -> Vec<i32> {
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
        | Instruction::I32Rotr => vec![-2, 1],
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
        | Instruction::I32GeU => vec![-2, 1],
        // i32 testop
        Instruction::I32Eqz => vec![-1, 1],
        // i32 unop
        Instruction::I32Clz | Instruction::I32Ctz | Instruction::I32Popcnt => vec![-1, 1],
        Instruction::I32Const(_) => vec![1],
        Instruction::GetLocal(_) => vec![1],
        Instruction::SetLocal(_) => vec![-1],
        Instruction::TeeLocal(_) => vec![-1, 2, -1],
        Instruction::Nop => vec![],
        _ => {
            if WHITELIST.contains(instr) {
                panic!("Forgot to implement instruction {}", instr);
            } else {
                panic!("Instruction {} not supported.", instr);
            }
        }
    }
}

pub fn get_equiv_instr<R: Rng + ?Sized>(rng: &mut R, instr: &Instruction) -> Instruction {
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
            match *LOCALOP.choose(rng).unwrap() {
                Instruction::GetLocal(_) => Instruction::GetLocal(i),
                Instruction::SetLocal(_) => Instruction::SetLocal(i),
                Instruction::TeeLocal(_) => Instruction::TeeLocal(i),
                _ => panic!("should never happen."),
            }
        }
        Instruction::Nop => Instruction::Nop,
        _ => {
            if WHITELIST.contains(instr) {
                panic!("Forgot to implement instruction {}", instr);
            } else {
                panic!("Instruction {} not supported.", instr);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn check_test() {
        check_instrs(&WHITELIST);

        for instr in &I32BINOP {
            assert!(&WHITELIST.contains(instr));
        }
        for instr in &I32UNOP {
            assert!(&WHITELIST.contains(instr));
        }
        for instr in &I32RELOP {
            assert!(&WHITELIST.contains(instr));
        }
        for instr in &LOCALOP {
            assert!(&WHITELIST.contains(instr));
        }

        for instr in &[
            Instruction::I32Eqz,
            Instruction::GetLocal(0),
            Instruction::Nop,
        ] {
            assert!(&WHITELIST.contains(instr));
        }
    }

    #[test]
    fn stack_cnt_whitelist_test() {
        // This should never panic
        for instr in WHITELIST.iter() {
            let _cnts = stack_cnt(instr);
        }
    }
}
