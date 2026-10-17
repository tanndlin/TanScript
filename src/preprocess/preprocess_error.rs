use std::{fmt, path::PathBuf};

use crate::error::{At, Spanned};

#[derive(Debug)]
pub enum PreprocessErrorKind {
    CouldNotRead { path: PathBuf, reason: String },
    UnknownItem { module: String, name: String },
    CircularImport(String),
    DuplicateDefinition(String),
}

impl At for PreprocessErrorKind {}

pub type PreprocessError = Spanned<PreprocessErrorKind>;

impl fmt::Display for PreprocessErrorKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            PreprocessErrorKind::CouldNotRead { path, reason } => {
                write!(f, "Could not read module ({}): {reason}", path.display())
            }
            PreprocessErrorKind::UnknownItem { module, name } => {
                write!(
                    f,
                    "Module ({module}) has no function or struct named ({name})"
                )
            }
            PreprocessErrorKind::CircularImport(module) => {
                write!(f, "Circular import of module ({module})")
            }
            PreprocessErrorKind::DuplicateDefinition(name) => {
                write!(f, "({name}) is defined more than once")
            }
        }
    }
}
