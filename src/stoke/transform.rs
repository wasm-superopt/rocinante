use crate::wasm::{Candidate, Whitelist};
use parity_wasm::elements::{Instruction, ValueType};
use rand::distributions::{Distribution, Standard};
use rand::seq::SliceRandom;
use rand::Rng;

#[derive(Debug, PartialEq, Eq, Copy, Clone)]
pub enum TransformKind {
    Opcode,
    Operand,
    Swap,
    Instruction,
    TwoInstructions,
}

impl Distribution<TransformKind> for Standard {
    fn sample<R: Rng + ?Sized>(&self, rng: &mut R) -> TransformKind {
        match rng.gen_range(0, 5) {
            0 => TransformKind::Opcode,
            1 => TransformKind::Operand,
            2 => TransformKind::Swap,
            3 => TransformKind::Instruction,
            _ => TransformKind::TwoInstructions,
        }
    }
}

pub struct TransformInfo {
    #[allow(dead_code)]
    success: bool,
    kind: TransformKind,
    undo_indices: [usize; 2],
    undo_instrs: [Instruction; 2],
}

pub struct Transform {
    spec_param_types: Vec<ValueType>,
    spec_local_types: Vec<ValueType>,
}

impl Transform {
    pub fn new(spec_param_types: Vec<ValueType>, spec_local_types: Vec<ValueType>) -> Self {
        Self {
            spec_param_types,
            spec_local_types,
        }
    }

    fn do_transform<R: Rng + ?Sized>(
        &self,
        rng: &mut R,
        transform_kind: TransformKind,
        instr_whitelist: &Whitelist,
        candidate_func: &mut Candidate,
    ) -> TransformInfo {
        match transform_kind {
            TransformKind::Opcode => self.opcode(rng, instr_whitelist, candidate_func),
            TransformKind::Operand => self.operand(rng, instr_whitelist, candidate_func),
            TransformKind::Swap => self.swap(rng, candidate_func),
            TransformKind::Instruction => self.instruction(rng, instr_whitelist, candidate_func),
            TransformKind::TwoInstructions => {
                self.two_instructions(rng, instr_whitelist, candidate_func)
            }
        }
    }

    pub fn operate<R: Rng + ?Sized>(
        &self,
        rng: &mut R,
        instr_whitelist: &Whitelist,
        candidate_func: &mut Candidate,
    ) -> TransformInfo {
        let transform_kind = rng.gen::<TransformKind>();

        self.do_transform(rng, transform_kind, instr_whitelist, candidate_func)
    }

    pub fn undo(&self, transform_info: &TransformInfo, candidate_func: &mut Candidate) {
        match transform_info.kind {
            TransformKind::Opcode | TransformKind::Operand | TransformKind::Instruction => {
                candidate_func.instrs_mut()[transform_info.undo_indices[0]] =
                    transform_info.undo_instrs[0].clone();
            }
            TransformKind::TwoInstructions => {
                // Must undo the second instruction first in case
                // both instruction transformations affected the
                // same index
                candidate_func.instrs_mut()[transform_info.undo_indices[1]] =
                    transform_info.undo_instrs[1].clone();
                candidate_func.instrs_mut()[transform_info.undo_indices[0]] =
                    transform_info.undo_instrs[0].clone();
            }
            TransformKind::Swap => {
                candidate_func.instrs_mut().swap(
                    transform_info.undo_indices[0],
                    transform_info.undo_indices[1],
                );
            }
        }
    }

    fn opcode<R: Rng + ?Sized>(
        &self,
        rng: &mut R,
        instr_whitelist: &Whitelist,
        candidate_func: &mut Candidate,
    ) -> TransformInfo {
        let (idx, undo_instr) = candidate_func.get_rand_instr(rng);
        let new_instr = instr_whitelist.get_equiv_instr(rng, &undo_instr);

        let instrs = candidate_func.instrs_mut();
        instrs[idx] = new_instr.clone();

        TransformInfo {
            success: new_instr != undo_instr,
            kind: TransformKind::Opcode,
            undo_indices: [idx, 0],
            undo_instrs: [undo_instr, parity_wasm::elements::Instruction::Nop],
        }
    }

    fn get_equiv_local_idx<R: Rng + ?Sized>(&self, rng: &mut R, idx: u32) -> u32 {
        let i = idx as usize;
        let typ: &ValueType = if i < self.spec_param_types.len() {
            &self.spec_param_types[i]
        } else if i < self.spec_param_types.len() + self.spec_local_types.len() {
            &self.spec_local_types[i - self.spec_param_types.len()]
        } else {
            panic!("local index out of bounds: {}", i);
        };

        let mut equiv_indices = Vec::new();
        for (i, param_type) in self.spec_param_types.iter().enumerate() {
            if param_type == typ {
                equiv_indices.push(i);
            }
        }

        for (i, local_type) in self.spec_local_types.iter().enumerate() {
            if local_type == typ {
                equiv_indices.push(i + self.spec_param_types.len());
            }
        }

        assert!(!equiv_indices.is_empty());

        *equiv_indices.choose(rng).unwrap() as u32
    }

    fn operand<R: Rng + ?Sized>(
        &self,
        rng: &mut R,
        instr_whitelist: &Whitelist,
        candidate_func: &mut Candidate,
    ) -> TransformInfo {
        let (instr_idx, undo_instr) = candidate_func.get_rand_instr(rng);

        let new_instr: Instruction = match &undo_instr {
            Instruction::GetLocal(i) => Instruction::GetLocal(self.get_equiv_local_idx(rng, *i)),
            Instruction::SetLocal(i) => Instruction::SetLocal(self.get_equiv_local_idx(rng, *i)),
            Instruction::TeeLocal(i) => Instruction::SetLocal(self.get_equiv_local_idx(rng, *i)),
            Instruction::I32Const(_) => {
                Instruction::I32Const(instr_whitelist.sample_i32_const(rng))
            }
            _ => {
                if instr_whitelist.is_instr_whitelisted(&undo_instr) {
                    undo_instr.clone()
                } else {
                    panic!("Instruction not implemented.")
                }
            }
        };

        let instrs = candidate_func.instrs_mut();
        instrs[instr_idx] = new_instr.clone();

        TransformInfo {
            success: new_instr != undo_instr,
            kind: TransformKind::Operand,
            undo_indices: [instr_idx, 0],
            undo_instrs: [undo_instr, parity_wasm::elements::Instruction::Nop],
        }
    }

    fn swap<R: Rng + ?Sized>(&self, rng: &mut R, candidate_func: &mut Candidate) -> TransformInfo {
        let (idx1, instr1) = candidate_func.get_rand_instr(rng);
        let (idx2, instr2) = candidate_func.get_rand_instr(rng);

        candidate_func.instrs_mut().swap(idx1, idx2);

        TransformInfo {
            success: idx1 != idx2 && instr1 != instr2,
            kind: TransformKind::Swap,
            undo_indices: [idx1, idx2],
            undo_instrs: [
                parity_wasm::elements::Instruction::Nop,
                parity_wasm::elements::Instruction::Nop,
            ],
        }
    }

    fn instruction<R: Rng + ?Sized>(
        &self,
        rng: &mut R,
        instr_whitelist: &Whitelist,
        candidate_func: &mut Candidate,
    ) -> TransformInfo {
        let (instr_idx, undo_instr) = candidate_func.get_rand_instr(rng);
        let new_instr: Instruction = instr_whitelist.sample(rng);

        let instrs = candidate_func.instrs_mut();
        instrs[instr_idx] = new_instr.clone();

        TransformInfo {
            success: new_instr != undo_instr,
            kind: TransformKind::Instruction,
            undo_indices: [instr_idx, 0],
            undo_instrs: [undo_instr, parity_wasm::elements::Instruction::Nop],
        }
    }
    fn two_instructions<R: Rng + ?Sized>(
        &self,
        rng: &mut R,
        instr_whitelist: &Whitelist,
        candidate_func: &mut Candidate,
    ) -> TransformInfo {
        let first_transform = self.instruction(rng, instr_whitelist, candidate_func);

        let second_transform = self.instruction(rng, instr_whitelist, candidate_func);

        let idx1 = first_transform.undo_indices[0];
        let idx2 = second_transform.undo_indices[0];
        let undo_instr1 = first_transform.undo_instrs[0].clone();
        let undo_instr2 = second_transform.undo_instrs[0].clone();
        TransformInfo {
            success: first_transform.success && second_transform.success && idx1 != idx2,
            kind: TransformKind::TwoInstructions,
            undo_indices: [idx1, idx2],
            undo_instrs: [undo_instr1, undo_instr2],
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use parity_wasm::elements::{Instruction, ValueType};
    #[test]
    fn opcode_transform_test() {
        let transform = Transform::new(vec![ValueType::I32], vec![]);
        let instr_whitelist = Whitelist::new(1, 0, &[1]);

        let original = Candidate::from_instrs(vec![Instruction::Nop, Instruction::I32Const(1)]);

        let mut transformed = original.clone();
        let transform_info = transform.do_transform(
            &mut rand::thread_rng(),
            TransformKind::Opcode,
            &instr_whitelist,
            &mut transformed,
        );

        if transform_info.success {
            assert_ne!(transformed.instrs(), original.instrs());
            println!("{:?}", transformed);
            println!("{:?}", original);
        }

        transform.undo(&transform_info, &mut transformed);
        assert_eq!(transformed.instrs(), original.instrs());
        println!("{:?}", transformed);
        println!("{:?}", original);
    }

    #[test]
    fn operand_transform_test() {
        let transform = Transform::new(vec![ValueType::I32], vec![]);
        let instr_whitelist = Whitelist::new(1, 0, &[1]);

        let original =
            Candidate::from_instrs(vec![Instruction::GetLocal(0), Instruction::I32Const(1)]);

        let mut transformed = original.clone();
        let transform_info = transform.do_transform(
            &mut rand::thread_rng(),
            TransformKind::Operand,
            &instr_whitelist,
            &mut transformed,
        );

        if transform_info.success {
            assert_ne!(transformed.instrs(), original.instrs());
            println!("{:?}", transformed);
            println!("{:?}", original);
        }

        transform.undo(&transform_info, &mut transformed);
        assert_eq!(transformed.instrs(), original.instrs());
        println!("{:?}", transformed);
        println!("{:?}", original);
    }

    #[test]
    fn swap_transform_test() {
        let transform = Transform::new(vec![ValueType::I32], vec![]);
        let instr_whitelist = Whitelist::new(1, 0, &[1]);

        let original =
            Candidate::from_instrs(vec![Instruction::GetLocal(0), Instruction::I32Const(1)]);

        let mut transformed = original.clone();
        let transform_info = transform.do_transform(
            &mut rand::thread_rng(),
            TransformKind::Swap,
            &instr_whitelist,
            &mut transformed,
        );

        if transform_info.success {
            assert_ne!(transformed.instrs(), original.instrs());
            println!("{:?}", transformed);
            println!("{:?}", original);
        }

        transform.undo(&transform_info, &mut transformed);
        assert_eq!(transformed.instrs(), original.instrs());
        println!("{:?}", transformed);
        println!("{:?}", original);
    }
}
