use crate::stoke::whitelist;
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
    success: bool,
    undo_indices: [usize; 2],
    undo_instr: Instruction,
}

pub trait Transform {
    fn new() -> Self;
    fn kind(&self) -> TransformKind;
    fn operate<R: Rng>(&self, rng: &mut R, instrs: &mut [Instruction]) -> TransformInfo;
    fn undo(&self, transform_info: &TransformInfo, instrs: &mut [Instruction]);
}

pub struct OpcodeTransform {}

impl Transform for OpcodeTransform {
    fn new() -> Self {
        OpcodeTransform {}
    }
    fn kind(&self) -> TransformKind {
        TransformKind::Opcode
    }

    fn operate<R: Rng>(&self, rng: &mut R, instrs: &mut [Instruction]) -> TransformInfo {
        let mut ti = TransformInfo {
            success: false,
            undo_indices: [0, 0],
            undo_instr: parity_wasm::elements::Instruction::Nop,
        };

        let idx: usize = rng.gen_range(0, instrs.len());
        ti.undo_indices[0] = idx;
        ti.undo_instr = instrs[idx].clone();

        let new_instr = whitelist::get_equiv_instr(rng, &ti.undo_instr);
        ti.success = new_instr != ti.undo_instr;
        instrs[idx] = new_instr;

        ti
    }

    fn undo(&self, transform_info: &TransformInfo, instrs: &mut [Instruction]) {
        instrs[transform_info.undo_indices[0]] = transform_info.undo_instr.clone();
    }
}

#[cfg(test)]
mod test {
    use super::*;
    #[test]
    fn opcode_transform_test() {
        let transform = OpcodeTransform::new();
        assert_eq!(transform.kind(), TransformKind::Opcode);
        let original_instrs = vec![
            Instruction::I32Add,
            Instruction::I32Or,
            Instruction::End,
            Instruction::Nop,
            Instruction::GetLocal(3),
        ];

        let mut transformed = original_instrs.clone();
        let transform_info = transform.operate(&mut rand::thread_rng(), &mut transformed);

        if transform_info.success {
            assert_ne!(transformed, original_instrs);
            println!("{:?}", transformed);
            println!("{:?}", original_instrs);
        }

        transform.undo(&transform_info, &mut transformed);
        assert_eq!(transformed, original_instrs);
        println!("{:?}", transformed);
        println!("{:?}", original_instrs);
    }
}
