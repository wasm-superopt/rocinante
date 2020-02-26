use crate::stoke::Candidate;
use parity_wasm::elements::Instruction;
use rand::seq::SliceRandom;
use rand::Rng;

pub fn sample<R: Rng + ?Sized>(
    rng: &mut R,
    // TODO(taegyunkim): Support increasing the number of locals.
    candidate_func: &mut Candidate,
) -> Instruction {
    match rng.gen_range(0, 21) {
        0 => Instruction::I32Add,
        1 => Instruction::I32Sub,
        2 => Instruction::I32Mul,
        3 => Instruction::I32DivS,
        4 => Instruction::I32DivU,
        5 => Instruction::I32RemS,
        6 => Instruction::I32RemU,
        7 => Instruction::I32And,
        8 => Instruction::I32Or,
        9 => Instruction::I32Xor,
        10 => Instruction::I32Shl,
        11 => Instruction::I32ShrS,
        12 => Instruction::I32ShrU,
        13 => Instruction::I32Rotl,
        14 => Instruction::I32Rotr,
        15 => Instruction::I32Const(candidate_func.sample_i32(rng)),
        16 => Instruction::GetLocal(candidate_func.sample_local_idx(rng)),
        17 => Instruction::SetLocal(candidate_func.sample_local_idx(rng)),
        18 => Instruction::TeeLocal(candidate_func.sample_local_idx(rng)),
        19 => Instruction::I32LeU,
        _ => Instruction::Nop,
    }
}

pub fn validate(_instrs: &[Instruction]) {
    // for instr in instrs {
    //     // TODO(taegyunkim): Handle control flow instructions separately.
    //     if *instr == Instruction::End {
    //         continue;
    //     }
    //     let _: Instruction = instr.clone().into();
    // }
}

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

const VAROP: [fn(n: u32) -> Instruction; 3] = [
    Instruction::GetLocal,
    Instruction::SetLocal,
    Instruction::TeeLocal,
    // Instruction::GetGlobal,
    // Instruction::SetGlobal,
];

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
        | Instruction::I32Rotr
        | Instruction::I32LeU => I32BINOP.choose(rng).unwrap().clone(),
        Instruction::GetLocal(i) | Instruction::SetLocal(i) | Instruction::TeeLocal(i) => {
            (*VAROP.choose(rng).unwrap())(i)
        }
        Instruction::I32Const(i) => Instruction::I32Const(i),
        Instruction::Nop => Instruction::Nop,
        _ => panic!("not implemented."),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use parity_wasm::elements::Instruction;

    #[test]
    fn validate_success_test() {
        validate(&vec![
            Instruction::I32Add,
            Instruction::I32Sub,
            Instruction::I32Mul,
            Instruction::I32DivU,
            Instruction::I32DivS,
            Instruction::I32RemU,
            Instruction::I32RemS,
            Instruction::I32And,
            Instruction::I32Or,
            Instruction::I32Xor,
            Instruction::I32Shl,
            Instruction::I32ShrU,
            Instruction::I32ShrS,
            Instruction::I32Rotl,
            Instruction::I32Rotr,
            Instruction::I32LeU,
            Instruction::I32Const(1),
            Instruction::GetLocal(2),
            Instruction::SetLocal(3),
            Instruction::TeeLocal(4),
            Instruction::Nop,
        ]);
    }
}
