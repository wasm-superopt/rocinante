use parity_wasm::elements::{FuncBody, FunctionType, Instruction};
use rand::distributions::{Distribution, Standard};
use rand::seq::SliceRandom;
use rand::Rng;

#[allow(dead_code)]
const I32UNOP: [Instruction; 3] = [
    Instruction::I32Clz,
    Instruction::I32Ctz,
    Instruction::I32Popcnt,
];

const I32BINOP: [Instruction; 7] = [
    Instruction::I32Add,
    Instruction::I32Sub,
    Instruction::I32Mul,
    Instruction::I32DivS,
    Instruction::I32DivU,
    Instruction::I32RemS,
    Instruction::I32RemU,
    // Instruction::I32And,
    // Instruction::I32Or,
    // Instruction::I32Xor,
    // Instruction::I32Shl,
    // Instruction::I32ShrS,
    // Instruction::I32ShrU,
    // Instruction::I32Rotl,
    // Instruction::I32Rotr,
];

#[allow(dead_code)]
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

#[allow(dead_code)]
const VAROP: [fn(n: u32) -> Instruction; 3] = [
    Instruction::GetLocal,
    Instruction::SetLocal,
    Instruction::TeeLocal,
    // Instruction::GetGlobal,
    // Instruction::SetGlobal,
];

pub fn get_equiv<R: Rng>(rng: &mut R, instr: &Instruction) -> Instruction {
    match instr {
        // _ if I32UNOP.contains(instr) => I32UNOP.choose(rng).unwrap().clone(),
        _ if I32BINOP.contains(instr) => I32BINOP.choose(rng).unwrap().clone(),
        // _ if I32RELOP.contains(instr) => I32RELOP.choose(rng).unwrap().clone(),
        // Instruction::I32Eqz => Instruction::I32Eqz,
        Instruction::End => Instruction::End,
        _ => panic!("get_equiv not implemented for {}", instr),
    }
}

#[derive(Debug)]
pub enum Transform {
    Opcode,
    Operand,
    Swap,
    Instruction,
}

impl Distribution<Transform> for Standard {
    fn sample<R: Rng + ?Sized>(&self, rng: &mut R) -> Transform {
        match rng.gen_range(0, 4) {
            0 => Transform::Opcode,
            1 => Transform::Operand,
            2 => Transform::Swap,
            _ => Transform::Instruction,
        }
    }
}

pub fn do_transform(_func_type: &FunctionType, _func_body: &mut FuncBody) {
    let transform: Transform = rand::random();

    match transform {
        Transform::Opcode => {
            // Choose an instruction at random, and replace with a random,
            // equivalent one.
        }
        Transform::Operand => {
            // Select an instruction at random, and its operand is replaced by a
            // random operand drawn from an equivalence class of operands.
        }
        Transform::Swap => {
            // Select two instructions from the set of original instructions
            // union with Nop, and swap
        }
        Transform::Instruction => {
            // Select an instruction, and replace with a random instruction,
            // with random operands.
        }
    }
}
