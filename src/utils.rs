use wasmi::{Error, ExternVal, FuncRef, ModuleInstance};

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

pub fn to_etype(val_type: wasmi::ValueType) -> parity_wasm::elements::ValueType {
    match val_type {
        wasmi::ValueType::I32 => parity_wasm::elements::ValueType::I32,
        wasmi::ValueType::I64 => parity_wasm::elements::ValueType::I64,
        wasmi::ValueType::F32 => parity_wasm::elements::ValueType::F32,
        wasmi::ValueType::F64 => parity_wasm::elements::ValueType::F64,
    }
}

pub fn to_func_type(signature: &wasmi::Signature) -> parity_wasm::elements::FunctionType {
    let param_types = signature
        .params()
        .to_vec()
        .into_iter()
        .map(to_etype)
        .collect();
    let return_type = signature.return_type().map(to_etype);
    parity_wasm::elements::FunctionType::new(param_types, return_type)
}
