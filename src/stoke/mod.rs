use crate::{debug, exec, parity_wasm_utils, solver};
use parity_wasm::elements::{FuncBody, FunctionType, Instruction, Instructions, Internal, Module};
use rand::Rng;

pub use self::transform::*;
mod transform;

#[allow(dead_code)]
pub struct Superoptimizer {
    module: Module,
}

impl Superoptimizer {
    pub fn new(module: Module) -> Self {
        Superoptimizer { module }
    }

    pub fn run(&self) {}

    /// Finds a module that has functions equivalent to the functions in the given module.
    pub fn synthesize(&self, rng: &mut impl Rng) {
        // Module in wasmi, WASM interpreter. Instantiate this here and pass
        // down to exec module functions to avoid re-instantiation.
        let wasmi_module = wasmi::Module::from_parity_wasm_module(self.module.clone())
            .expect("Failed to load parity-wasm Module.");
        let instance = wasmi::ModuleInstance::new(&wasmi_module, &wasmi::ImportsBuilder::default())
            .expect("Failed to instantiate wasm module.")
            .assert_no_start();

        let export_section = self
            .module
            .export_section()
            .expect("Module doesn't have export section.");

        for export_entry in export_section.entries() {
            if let Internal::Function(_idx) = export_entry.internal() {
                let func_name = export_entry.field();

                let test_cases = exec::generate_test_cases(rng, &instance, func_name);
                // let _generator = Generator::new(&func_type);
                let (func_type, func_body) =
                    parity_wasm_utils::func_by_name(&self.module, func_name);

                let cfg = z3::Config::new();
                let ctx = z3::Context::new(&cfg);
                let z3solver = solver::Z3Solver::new(&ctx, func_type, func_body);
                let mut generator = Generator::new(func_type);

                loop {
                    let module = generator.module();
                    if exec::eval_test_cases(module.clone(), &test_cases) > 0 {
                        generator.do_transform(rng);
                        continue;
                    }
                    match z3solver.verify(generator.get_candidate_func()) {
                        solver::VerifyResult::Verified => {
                            // collect the function from generator
                            debug::print_functions(&module);
                            break;
                        }
                        solver::VerifyResult::CounterExample(_) => {
                            // Add input, output pair to the test cases.
                            generator.do_transform(rng)
                        }
                    }
                }
            }
        }
    }
}

pub struct Generator {
    func_type: FunctionType,
    func_body: FuncBody,
}

impl Generator {
    pub fn new(func_type: &FunctionType) -> Self {
        let func_body = FuncBody::new(vec![], Instructions::empty());
        Self {
            func_type: func_type.clone(),
            func_body,
        }
    }

    // fn get_local_types(&self, param_types: &[ValueType], locals: &[Local]) -> Vec<ValueType> {
    //     let mut types = param_types.to_vec();

    //     for local in locals {
    //         let count = local.count() as usize;
    //         types.reserve(count);
    //         let local_type = local.value_type();
    //         for _ in 0..count {
    //             types.push(local_type);
    //         }
    //     }

    //     types
    // }

    pub fn do_transform<R: Rng>(&mut self, rng: &mut R) {
        let transform: Transform = rng.gen::<Transform>();
        let instrs = self.func_body.code().elements();

        match transform {
            Transform::Opcode => {
                // Choose an instruction at random, and replace with a random,
                // equivalent one.
                let idx: usize = rng.gen_range(0, instrs.len());
                let chosen_instr = &instrs[idx];
                let new_instr = get_equiv(rng, chosen_instr);
                let mut new_instrs = Vec::with_capacity(instrs.len());
                new_instrs.clone_from_slice(instrs);
                new_instrs[idx] = new_instr;
            }
            Transform::Operand => {
                // Select an instruction at random, and its operand is replaced by a
                // random operand drawn from an equivalence class of operands.
                let idx: usize = rng.gen_range(0, instrs.len());
                let chosen_instr = &instrs[idx];

                match chosen_instr {
                    Instruction::GetLocal(_)
                    | Instruction::SetLocal(_)
                    | Instruction::TeeLocal(_) => {}
                    Instruction::GetGlobal(_) | Instruction::SetGlobal(_) => {}
                    _ => {}
                }
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

    pub fn module(&self) -> Module {
        parity_wasm_utils::build_module("candidate", &self.func_type, self.func_body.clone())
    }

    pub fn get_candidate_func(&self) -> &FuncBody {
        &self.func_body
    }
}
