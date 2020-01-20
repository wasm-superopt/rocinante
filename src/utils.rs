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
