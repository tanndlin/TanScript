use crate::{ast::FunctionDefinition, type_check::Type};

#[derive(Debug, Clone)]
pub struct FunctionContext {
    pub epilogue_label: String,
    pub return_type: Type,
}

impl From<&FunctionDefinition> for FunctionContext {
    fn from(def: &FunctionDefinition) -> Self {
        Self {
            epilogue_label: format!("{}_epilogue", def.name),
            return_type: def.return_type.clone(),
        }
    }
}
