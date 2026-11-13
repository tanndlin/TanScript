use crate::{
    ast::{
        Arg, AtomType, Child, ExpressionKind, Signature, Statement, StatementOrExpression,
        StructField, Walk,
    },
    type_check::Type,
};

pub trait References {
    fn references(&self, names: &mut Vec<String>);
}

impl<T: References> References for Option<T> {
    fn references(&self, names: &mut Vec<String>) {
        if let Some(item) = self {
            item.references(names);
        }
    }
}

impl<T: References> References for [T] {
    fn references(&self, names: &mut Vec<String>) {
        for item in self {
            item.references(names);
        }
    }
}

impl References for Arg {
    fn references(&self, names: &mut Vec<String>) {
        self.atype.references(names);
    }
}

impl References for Signature {
    fn references(&self, names: &mut Vec<String>) {
        self.args.references(names);
        self.return_type.references(names);
    }
}

impl References for StructField {
    fn references(&self, names: &mut Vec<String>) {
        self.ftype.references(names);
    }
}

impl StatementOrExpression {
    pub fn references(&self, names: &mut Vec<String>) {
        self.visit(&mut |c| {
            match c {
                Child::Statement(Statement::Declaration(s)) => {
                    s.dtype.references(names);
                }
                Child::Statement(Statement::FunctionDefinition(s)) => {
                    s.signature.references(names);
                }
                Child::Statement(Statement::ExternDeclaration(s)) => {
                    s.signature.references(names);
                }
                Child::Statement(Statement::StructDefinition(s)) => {
                    s.fields.references(names);
                }
                Child::Expression(e) => match &e.kind {
                    ExpressionKind::Atom(AtomType::Identifier(name))
                    | ExpressionKind::FunctionCall(name, _)
                    | ExpressionKind::EnumVariant {
                        enum_name: name, ..
                    }
                    | ExpressionKind::StructInstantiation {
                        struct_type_name: name,
                        ..
                    } => names.push(name.clone()),
                    ExpressionKind::Cast(_, t) | ExpressionKind::SizeOf(t) => t.references(names),
                    _ => {}
                },
                _ => {}
            }
            true
        });
    }
}

impl References for Type {
    fn references(&self, names: &mut Vec<String>) {
        match self {
            Type::Named {
                name,
                concrete_generics,
                ..
            } => {
                names.push(name.clone());
                concrete_generics.references(names);
            }
            Type::Struct(s) => names.push(s.name.clone()),
            Type::Enum(e) => names.push(e.name.clone()),
            Type::Pointer(inner) => inner.references(names),
            Type::Function(f) => {
                f.args.references(names);
                f.return_type.references(names);
            }
            Type::Int(_)
            | Type::Char
            | Type::Boolean
            | Type::Unit
            | Type::Null
            | Type::Unknown
            | Type::Generic(_) => {}
        }
    }
}
