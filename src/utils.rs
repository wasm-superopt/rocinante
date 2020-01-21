use parity_wasm::elements::{
    Instruction as EInstruction, Instructions as EInstructions, Module as EModule,
    ValueType as EValueType,
};
use wasmi::{Error, ExternVal, FuncRef, ModuleInstance, Signature, ValueType};

/// Copied from wasmi::ModuleInstance::func_by_name, as the function is private.
pub fn func_by_name(instance: &ModuleInstance, func_name: &str) -> Result<FuncRef, Error> {
    let extern_val = instance
        .export_by_name(func_name)
        .ok_or_else(|| Error::Function(format!("Module doesn't have export {}", func_name)))?;

    match extern_val {
        ExternVal::Func(func_instance) => Ok(func_instance),
        unexpected => Err(Error::Function(format!(
            "Export {} is not a function, but {:?}",
            func_name, unexpected
        ))),
    }
}

fn to_etype(val_type: wasmi::ValueType) -> EValueType {
    match val_type {
        ValueType::I32 => EValueType::I32,
        ValueType::I64 => EValueType::I64,
        ValueType::F32 => EValueType::F32,
        ValueType::F64 => EValueType::F64,
    }
}

pub fn gen_random_func(signature: &Signature) -> EModule {
    let param_types: Vec<EValueType> = signature
        .params()
        .iter()
        .map(|val_type| to_etype(*val_type))
        .collect();
    let return_type: Option<EValueType> = match &signature.return_type() {
        Some(val_type) => Some(to_etype(*val_type)),
        None => None,
    };

    let instr: EInstruction = match return_type {
        None => EInstruction::End,
        Some(val_type) => match val_type {
            EValueType::I32 => EInstruction::I32Const(0),
            EValueType::I64 => EInstruction::I64Const(0),
            EValueType::F32 => EInstruction::F32Const(0),
            EValueType::F64 => EInstruction::F64Const(0),
        },
    };

    #[rustfmt::skip]
    let module = parity_wasm::builder::module()
        .export()
            .field("candidate")
            .internal()
            .func(0)
            .build()
        .function()
            .signature()
                .with_params(param_types)
                .with_return_type(return_type)
                .build()
            .body()
                .with_instructions(EInstructions::new(vec![instr]))
                .build()
            .build()
        .build();

    module
}
