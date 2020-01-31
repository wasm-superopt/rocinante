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

pub struct Transform {
    kind: TransformKind,
}

impl Distribution<Transform> for Standard {
    fn sample<R: Rng + ?Sized>(&self, rng: &mut R) -> Transform {
        Transform::new(rng.gen::<TransformKind>())
    }
}

impl Transform {
    pub fn new(kind: TransformKind) -> Self {
        Self { kind }
    }

    pub fn kind(&self) -> TransformKind {
        self.kind
    }

    pub fn operate(
        &self,
        rng: &mut rand::rngs::ThreadRng,
        candidate_func: &mut CandidateFunc,
    ) -> TransformInfo {
        match self.kind() {
            TransformKind::Opcode => self.opcode(rng, candidate_func),
            unimplemented => {
                panic!("Unimplemented: {:?}", unimplemented);
            }
        }
    }

    pub fn undo(&self, transform_info: &TransformInfo, candidate_func: &mut CandidateFunc) {
        candidate_func.instrs_mut()[transform_info.undo_indices[0]] =
            transform_info.undo_instr.clone();
    }

    fn opcode(
        &self,
        rng: &mut rand::rngs::ThreadRng,
        candidate_func: &mut CandidateFunc,
    ) -> TransformInfo {
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
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::stoke::CandidateFunc;
    use parity_wasm::elements::{FunctionType, ValueType};
    #[test]
    fn opcode_transform_test() {
        let transform = Transform::new(TransformKind::Opcode);
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
