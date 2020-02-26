use crate::stoke::whitelist;
use crate::stoke::Candidate;
use parity_wasm::elements::Instruction;
use rand::distributions::{Distribution, Standard};
use rand::Rng;
use whitelist::WhitelistedInstruction;

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

pub fn stack_cnt(instr: &Instruction) -> i32 {
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
        | Instruction::I32LeU => -1,
        Instruction::I32Const(_) | Instruction::GetLocal(_) => 1,
        Instruction::SetLocal(_) => -1,
        Instruction::TeeLocal(_) => 1,
        Instruction::Nop => 0,
        _ => panic!("instruction {}, unimplemented", instr),
    }
}

impl Transform {
    pub fn new(kind: TransformKind) -> Self {
        Self { kind }
    }

    pub fn kind(&self) -> TransformKind {
        self.kind
    }

    pub fn operate<R: Rng + ?Sized>(
        &self,
        rng: &mut R,
        candidate_func: &mut Candidate,
    ) -> TransformInfo {
        match self.kind() {
            TransformKind::Opcode => self.opcode(rng, candidate_func),
            TransformKind::Operand => self.operand(rng, candidate_func),
            TransformKind::Swap => self.swap(rng, candidate_func),
            TransformKind::Instruction => self.instruction(rng, candidate_func),
        }
    }

    pub fn undo(&self, transform_info: &TransformInfo, candidate_func: &mut Candidate) {
        match transform_info.kind {
            TransformKind::Opcode | TransformKind::Operand | TransformKind::Instruction => {
                let current_instr =
                    candidate_func.instrs_mut()[transform_info.undo_indices[0]].clone();

                candidate_func.dec_stack_cnt(stack_cnt(&current_instr));
                candidate_func.inc_stack_cnt(stack_cnt(&transform_info.undo_instr));

                candidate_func.instrs_mut()[transform_info.undo_indices[0]] =
                    transform_info.undo_instr.clone();
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
        candidate_func: &mut Candidate,
    ) -> TransformInfo {
        let (idx, undo_instr) = candidate_func.get_rand_instr(rng);
        let new_instr = whitelist::get_equiv_instr(rng, &undo_instr);

        candidate_func.dec_stack_cnt(stack_cnt(&undo_instr));
        candidate_func.inc_stack_cnt(stack_cnt(&new_instr));

        let instrs = candidate_func.instrs_mut();
        instrs[idx] = new_instr.clone();

        TransformInfo {
            success: new_instr != undo_instr,
            kind: self.kind(),
            undo_indices: [idx, 0],
            undo_instr,
        }
    }

    fn operand<R: Rng + ?Sized>(
        &self,
        rng: &mut R,
        candidate_func: &mut Candidate,
    ) -> TransformInfo {
        let (instr_idx, undo_instr) = candidate_func.get_rand_instr(rng);

        // NOTE(taegyunkim): Force conversion to WhitelistedInstruction to avoid
        // a situation where we get a missing case leading to a bug.
        let new_instr: Instruction = match undo_instr.clone().into() {
            WhitelistedInstruction::GetLocal(i) => {
                WhitelistedInstruction::GetLocal(candidate_func.get_equiv_local_idx(rng, i))
            }
            WhitelistedInstruction::SetLocal(i) => {
                WhitelistedInstruction::SetLocal(candidate_func.get_equiv_local_idx(rng, i))
            }
            WhitelistedInstruction::TeeLocal(i) => {
                WhitelistedInstruction::SetLocal(candidate_func.get_equiv_local_idx(rng, i))
            }
            WhitelistedInstruction::I32Add => WhitelistedInstruction::I32Add,
            WhitelistedInstruction::I32Sub => WhitelistedInstruction::I32Sub,
            WhitelistedInstruction::I32Mul => WhitelistedInstruction::I32Mul,
            WhitelistedInstruction::I32DivS => WhitelistedInstruction::I32DivS,
            WhitelistedInstruction::I32DivU => WhitelistedInstruction::I32DivU,
            WhitelistedInstruction::I32RemS => WhitelistedInstruction::I32RemS,
            WhitelistedInstruction::I32RemU => WhitelistedInstruction::I32RemU,
            WhitelistedInstruction::I32And => WhitelistedInstruction::I32And,
            WhitelistedInstruction::I32Or => WhitelistedInstruction::I32Or,
            WhitelistedInstruction::I32Xor => WhitelistedInstruction::I32Xor,
            WhitelistedInstruction::I32Shl => WhitelistedInstruction::I32Shl,
            WhitelistedInstruction::I32ShrS => WhitelistedInstruction::I32ShrS,
            WhitelistedInstruction::I32ShrU => WhitelistedInstruction::I32ShrU,
            WhitelistedInstruction::I32Rotl => WhitelistedInstruction::I32Rotl,
            WhitelistedInstruction::I32Rotr => WhitelistedInstruction::I32Rotr,
            WhitelistedInstruction::I32LeU => WhitelistedInstruction::I32LeU,
            WhitelistedInstruction::Nop => WhitelistedInstruction::Nop,
            WhitelistedInstruction::I32Const(_) => {
                WhitelistedInstruction::I32Const(candidate_func.sample_i32(rng))
            }
        }
        .into();

        candidate_func.dec_stack_cnt(stack_cnt(&undo_instr));
        candidate_func.inc_stack_cnt(stack_cnt(&new_instr));

        let instrs = candidate_func.instrs_mut();
        instrs[instr_idx] = new_instr.clone();

        TransformInfo {
            success: new_instr != undo_instr,
            kind: self.kind(),
            undo_indices: [instr_idx, 0],
            undo_instr,
        }
    }

    fn swap<R: Rng + ?Sized>(&self, rng: &mut R, candidate_func: &mut Candidate) -> TransformInfo {
        let (idx1, instr1) = candidate_func.get_rand_instr(rng);
        let (idx2, instr2) = candidate_func.get_rand_instr(rng);

        candidate_func.instrs_mut().swap(idx1, idx2);

        TransformInfo {
            success: idx1 != idx2 && instr1 != instr2,
            kind: self.kind(),
            undo_indices: [idx1, idx2],
            undo_instr: parity_wasm::elements::Instruction::Nop,
        }
    }

    fn instruction<R: Rng + ?Sized>(
        &self,
        rng: &mut R,
        candidate_func: &mut Candidate,
    ) -> TransformInfo {
        let (instr_idx, undo_instr) = candidate_func.get_rand_instr(rng);
        let new_instr: Instruction = WhitelistedInstruction::sample(rng, candidate_func).into();

        candidate_func.dec_stack_cnt(stack_cnt(&undo_instr));
        candidate_func.inc_stack_cnt(stack_cnt(&new_instr));

        let instrs = candidate_func.instrs_mut();
        instrs[instr_idx] = new_instr.clone();

        TransformInfo {
            success: new_instr != undo_instr,
            kind: self.kind(),
            undo_indices: [instr_idx, 0],
            undo_instr,
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::stoke::Candidate;
    use parity_wasm::elements::{FuncBody, FunctionType, Instruction, Instructions, ValueType};
    #[test]
    fn opcode_transform_test() {
        let transform = Transform::new(TransformKind::Opcode);
        assert_eq!(transform.kind(), TransformKind::Opcode);

        let original = Candidate::new(
            &FunctionType::new(vec![ValueType::I32], Some(ValueType::I32)),
            &FuncBody::new(
                vec![],
                Instructions::new(vec![
                    Instruction::Nop,
                    Instruction::End,
                    Instruction::I32Const(1),
                ]),
            ),
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

    #[test]
    fn operand_transform_test() {
        let transform = Transform::new(TransformKind::Operand);
        assert_eq!(transform.kind(), TransformKind::Operand);

        let original = Candidate::new(
            &FunctionType::new(vec![ValueType::I32], Some(ValueType::I32)),
            &FuncBody::new(
                vec![],
                Instructions::new(vec![
                    Instruction::Nop,
                    Instruction::End,
                    Instruction::I32Const(1),
                ]),
            ),
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

    #[test]
    fn swap_transform_test() {
        let transform = Transform::new(TransformKind::Swap);
        assert_eq!(transform.kind(), TransformKind::Swap);

        let original = Candidate::new(
            &FunctionType::new(vec![ValueType::I32], Some(ValueType::I32)),
            &FuncBody::new(
                vec![],
                Instructions::new(vec![
                    Instruction::Nop,
                    Instruction::End,
                    Instruction::I32Const(1),
                ]),
            ),
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
