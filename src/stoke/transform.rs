use crate::stoke::whitelist;
use crate::stoke::CandidateFunc;
use parity_wasm::elements::Instruction;
use rand::distributions::{Distribution, Standard};
use rand::Rng;

#[derive(Debug, PartialEq, Eq, Copy, Clone)]
pub enum TransformKind {
    Opcode,
    Operand,
    Swap,
    Instruction,
}

impl Distribution<TransformKind> for Standard {
    fn sample<R: Rng + ?Sized>(&self, rng: &mut R) -> TransformKind {
        match rng.gen_range(0, 4) {
            0 => TransformKind::Opcode,
            1 => TransformKind::Operand,
            2 => TransformKind::Swap,
            _ => TransformKind::Instruction,
        }
    }
}

pub struct TransformInfo {
    #[allow(dead_code)]
    success: bool,
    #[allow(dead_code)]
    kind: TransformKind,
    undo_indices: [usize; 2],
    undo_instr: Instruction,
}

pub trait Transform {
    fn new() -> Self;
    fn kind(&self) -> TransformKind;
    fn operate<R: Rng>(&self, rng: &mut R, candidate_func: &mut CandidateFunc) -> TransformInfo;
    fn undo(&self, transform_info: &TransformInfo, instrs: &mut CandidateFunc);
}

pub struct OpcodeTransform {}

impl Transform for OpcodeTransform {
    fn new() -> Self {
        OpcodeTransform {}
    }
    fn kind(&self) -> TransformKind {
        TransformKind::Opcode
    }

    fn operate<R: Rng>(&self, rng: &mut R, candidate_func: &mut CandidateFunc) -> TransformInfo {
        let (idx, undo_instr) = candidate_func.get_rand_instr(rng);
        let instrs = candidate_func.instrs_mut();

        let new_instr = whitelist::get_equiv_instr(rng, &undo_instr);

        instrs[idx] = new_instr.clone();

        TransformInfo {
            success: new_instr != undo_instr,
            kind: self.kind(),
            undo_indices: [idx, 0],
            undo_instr,
        }
    }

    fn undo(&self, transform_info: &TransformInfo, candidate_func: &mut CandidateFunc) {
        candidate_func.instrs_mut()[transform_info.undo_indices[0]] =
            transform_info.undo_instr.clone();
    }
}

pub struct OperandTransform {}

impl Transform for OperandTransform {
    fn new() -> Self {
        Self {}
    }

    fn kind(&self) -> TransformKind {
        TransformKind::Operand
    }

    fn operate<R: Rng>(&self, rng: &mut R, candidate_func: &mut CandidateFunc) -> TransformInfo {
        let (idx, undo_instr) = candidate_func.get_rand_instr(rng);
        let instrs = candidate_func.instrs_mut();

        let new_instr = whitelist::get_equiv_instr(rng, &undo_instr);

        instrs[idx] = new_instr.clone();

        TransformInfo {
            success: new_instr != undo_instr,
            kind: self.kind(),
            undo_indices: [idx, 0],
            undo_instr,
        }
    }

    fn undo(&self, transform_info: &TransformInfo, candidate_func: &mut CandidateFunc) {
        candidate_func.instrs_mut()[transform_info.undo_indices[0]] =
            transform_info.undo_instr.clone();
    }
}
#[cfg(test)]
mod test {
    use super::*;
    use crate::stoke::CandidateFunc;
    use parity_wasm::elements::{FunctionType, ValueType};
    #[test]
    fn opcode_transform_test() {
        let transform = OpcodeTransform::new();
        assert_eq!(transform.kind(), TransformKind::Opcode);

        let original = CandidateFunc::new(
            &FunctionType::new(vec![ValueType::I32], Some(ValueType::I32)),
            vec![-2, -1, 0, 1, 2],
        );

        let mut transformed = original.clone();
        let transform_info = transform.operate(&mut rand::thread_rng(), &mut transformed);

        if transform_info.success {
            assert_ne!(transformed, original);
            println!("{:?}", transformed);
            println!("{:?}", original);
        }

        transform.undo(&transform_info, &mut transformed);
        assert_eq!(transformed, original);
        println!("{:?}", transformed);
        println!("{:?}", original);
    }
}
