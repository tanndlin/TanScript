use std::collections::HashSet;

use crate::{
    ast::{
        AssignOp, Assignment, AtomType, Block, Declaration, EnumDefinition, ExpressionKind,
        ForLoop, FunctionDefinition, IfStatement, MatchStatement, ReturnStatement, Statement,
        StatementOrExpression, StructDefinition, WhileLoop,
    },
    error::At,
    type_check::{
        FunctionType, Type, TypeCheckError, TypeCheckErrorKind, TypeScope, check_binary,
        check_types_expression, expect_type, narrow_expression,
    },
    types::IntType,
};

impl Block {
    pub fn check_types(&mut self, type_scope: &mut TypeScope) -> Result<(), TypeCheckError> {
        type_scope.push();
        let result = self.declare(type_scope).and_then(|()| {
            self.children.iter_mut().try_for_each(|child| match child {
                StatementOrExpression::Statement(statement) => match statement.as_mut() {
                    Statement::StructDefinition(_) | Statement::EnumDefinition(_) => Ok(()),
                    Statement::FunctionDefinition(s) => s.check_body(type_scope),
                    _ => child.check_types(type_scope),
                },
                StatementOrExpression::Expression(_) => child.check_types(type_scope),
            })
        });
        type_scope.pop();
        result
    }

    fn declare(&mut self, type_scope: &mut TypeScope) -> Result<(), TypeCheckError> {
        let mut structs = vec![];
        for child in &mut self.children {
            if let StatementOrExpression::Statement(statement) = child {
                match statement.as_mut() {
                    Statement::StructDefinition(s) => {
                        s.declare(type_scope)?;
                        structs.push(s);
                    }
                    Statement::EnumDefinition(s) => s.check_types(type_scope)?,
                    _ => {}
                }
            }
        }
        for i in value_dependency_order(&structs) {
            structs[i].check_types(type_scope)?;
        }

        for child in &mut self.children {
            if let StatementOrExpression::Statement(statement) = child
                && let Statement::FunctionDefinition(s) = statement.as_mut()
            {
                s.declare(type_scope)?;
            }
        }
        Ok(())
    }

    pub fn resolve_types(&mut self, type_scope: &TypeScope) {
        for child in &mut self.children {
            child.resolve_types(type_scope);
        }
    }

    pub fn always_returns(&self) -> bool {
        self.children.iter().any(|child| match child {
            StatementOrExpression::Statement(statement) => match statement.as_ref() {
                Statement::Return(_) => true,
                Statement::IfStatement(IfStatement {
                    block,
                    else_block: Some(else_block),
                    ..
                }) => block.always_returns() && else_block.always_returns(),
                _ => false,
            },
            StatementOrExpression::Expression(_) => false,
        })
    }
}

fn value_dependency_order(structs: &[&mut StructDefinition]) -> Vec<usize> {
    let mut visited = vec![false; structs.len()];
    let mut order = vec![];
    for i in 0..structs.len() {
        visit_value_dependencies(i, structs, &mut visited, &mut order);
    }
    order
}

fn visit_value_dependencies(
    i: usize,
    structs: &[&mut StructDefinition],
    visited: &mut [bool],
    order: &mut Vec<usize>,
) {
    if visited[i] {
        return;
    }
    visited[i] = true;

    let mut names = vec![];
    for field in &structs[i].fields {
        value_dependencies(&field.ftype, &mut names);
    }
    for name in names {
        if let Some(dep) = structs.iter().position(|s| s.name == name) {
            visit_value_dependencies(dep, structs, visited, order);
        }
    }
    order.push(i);
}

fn value_dependencies<'a>(t: &'a Type, names: &mut Vec<&'a str>) {
    if let Type::Named {
        name,
        concrete_generics,
        ..
    } = t
    {
        names.push(name);
        for generic in concrete_generics {
            value_dependencies(generic, names);
        }
    }
}

impl StatementOrExpression {
    pub fn check_types(&mut self, type_scope: &mut TypeScope) -> Result<(), TypeCheckError> {
        match self {
            StatementOrExpression::Statement(statement) => statement.check_types(type_scope),
            StatementOrExpression::Expression(expression) => {
                check_types_expression(expression, type_scope).map(|_| ())
            }
        }
    }

    pub fn resolve_types(&mut self, type_scope: &TypeScope) {
        if let StatementOrExpression::Statement(statement) = self {
            statement.resolve_types(type_scope);
        }
    }
}

impl Statement {
    pub fn check_types(&mut self, type_scope: &mut TypeScope) -> Result<(), TypeCheckError> {
        match self {
            Self::Assignment(s) => s.check_types(type_scope),
            Self::Declaration(s) => s.check_types(type_scope),
            Self::ForLoop(s) => s.check_types(type_scope),
            Self::FunctionDefinition(s) => s.check_types(type_scope),
            Self::StructDefinition(s) => {
                s.declare(type_scope)?;
                s.check_types(type_scope)
            }
            Self::IfStatement(s) => s.check_types(type_scope),
            Self::MatchStatement(s) => s.check_types(type_scope),
            Self::Return(s) => s.check_types(type_scope),
            Self::WhileLoop(s) => s.check_types(type_scope),
            Self::EnumDefinition(s) => s.check_types(type_scope),
            Self::Break(_) | Self::Continue(_) => Ok(()),
        }
    }

    pub fn resolve_types(&mut self, type_scope: &TypeScope) {
        match self {
            Self::Declaration(s) => s.resolve_types(type_scope),
            Self::ForLoop(s) => s.resolve_types(type_scope),
            Self::FunctionDefinition(s) => s.body.resolve_types(type_scope),
            Self::IfStatement(s) => s.resolve_types(type_scope),
            Self::MatchStatement(s) => s.resolve_types(type_scope),
            Self::WhileLoop(s) => s.block.resolve_types(type_scope),
            Self::Assignment(_)
            | Self::Return(_)
            | Self::Break(_)
            | Self::Continue(_)
            | Self::StructDefinition(_)
            | Self::EnumDefinition(_) => {}
        }
    }
}

impl Assignment {
    pub fn check_types(&mut self, type_scope: &mut TypeScope) -> Result<(), TypeCheckError> {
        let expected = check_types_expression(&mut self.lhs, type_scope)?;
        let got = match self.op {
            AssignOp::Assign => check_types_expression(&mut self.expression, type_scope)?,
            AssignOp::Compound(op) => {
                check_binary(op, &mut self.lhs, &mut self.expression, type_scope)?
            }
        };

        if expected == Type::Int(IntType::Coercible) && got.is_number_type() {
            narrow_expression(&self.lhs, got, type_scope);
            return Ok(());
        }
        expect_type(&self.expression, &expected, &got, type_scope)
    }
}

impl Declaration {
    pub fn check_types(&mut self, type_scope: &mut TypeScope) -> Result<(), TypeCheckError> {
        let ExpressionKind::Atom(AtomType::Identifier(ident)) = &self.assign.lhs.kind else {
            panic!("Declaration LHS can only be a identifier");
        };

        let got = check_types_expression(&mut self.assign.expression, type_scope)?;
        let Some(expected) = self
            .dtype
            .as_ref()
            .map(|t| type_scope.resolve(t))
            .transpose()?
        else {
            self.binding = Some(type_scope.insert(ident.clone(), got));
            return Ok(());
        };

        expect_type(&self.assign.expression, &expected, &got, type_scope)?;
        self.binding = Some(type_scope.insert(ident.clone(), expected));
        Ok(())
    }

    pub fn resolve_types(&mut self, type_scope: &TypeScope) {
        if let Some(id) = self.binding {
            self.dtype = Some(type_scope.binding(id).clone());
        }
    }
}

impl ForLoop {
    pub fn check_types(&mut self, type_scope: &mut TypeScope) -> Result<(), TypeCheckError> {
        type_scope.push();
        let result = self.init.check_types(type_scope).and_then(|()| {
            check_types_expression(&mut self.condition, type_scope)?;
            self.update.check_types(type_scope)?;
            self.block.check_types(type_scope)
        });
        type_scope.pop();
        result
    }

    pub fn resolve_types(&mut self, type_scope: &TypeScope) {
        self.init.resolve_types(type_scope);
        self.block.resolve_types(type_scope);
    }
}

impl FunctionDefinition {
    pub fn check_types(&mut self, type_scope: &mut TypeScope) -> Result<(), TypeCheckError> {
        self.declare(type_scope)?;
        self.check_body(type_scope)
    }

    fn declare(&mut self, type_scope: &mut TypeScope) -> Result<(), TypeCheckError> {
        for arg in &mut self.args {
            arg.atype = type_scope.resolve(&arg.atype)?;
        }
        self.return_type = type_scope.resolve(&self.return_type)?;

        type_scope.insert(
            self.name.clone(),
            Type::Function(Box::new(FunctionType {
                args: self.args.iter().map(|a| a.atype.clone()).collect(),
                return_type: self.return_type.clone(),
            })),
        );
        Ok(())
    }

    fn check_body(&mut self, type_scope: &mut TypeScope) -> Result<(), TypeCheckError> {
        type_scope.push();
        type_scope.push_function(self.return_type.clone());
        for arg in &self.args {
            type_scope.insert(arg.name.clone(), arg.atype.clone());
        }
        let result = self.body.check_types(type_scope);
        type_scope.pop_function();
        type_scope.pop();
        result?;

        if self.return_type != Type::Unit && !self.body.always_returns() {
            return Err(TypeCheckErrorKind::TypeMismatch {
                expected: Box::new(self.return_type.clone()),
                got: Box::new(Type::Unit),
            }
            .at(self.span));
        }
        Ok(())
    }
}

impl EnumDefinition {
    pub fn check_types(&self, type_scope: &mut TypeScope) -> Result<(), TypeCheckError> {
        if type_scope.lookup(&self.name).is_some() {
            return Err(TypeCheckErrorKind::DuplicateIdentifiers(self.name.clone()).at(self.span));
        }

        for (i, variant) in self.variants.iter().enumerate() {
            if self.variants[..i].contains(variant) {
                return Err(TypeCheckErrorKind::DuplicateIdentifiers(variant.clone()).at(self.span));
            }
        }

        type_scope.insert(self.name.clone(), Type::Enum(Box::new(self.clone())));
        Ok(())
    }
}

impl StructDefinition {
    pub fn declare(&self, type_scope: &mut TypeScope) -> Result<(), TypeCheckError> {
        if type_scope.lookup(&self.name).is_some() {
            return Err(TypeCheckErrorKind::DuplicateIdentifiers(self.name.clone()).at(self.span));
        }

        type_scope.insert(
            self.name.clone(),
            Type::Named {
                name: self.name.clone(),
                concrete_generics: vec![],
                span: self.span,
            },
        );
        Ok(())
    }

    pub fn check_types(&mut self, type_scope: &mut TypeScope) -> Result<(), TypeCheckError> {
        type_scope.push();
        for generic in &self.generics {
            type_scope.insert(generic.clone(), Type::Generic(generic.clone()));
        }

        let mut parent_structs = HashSet::from([self.name.clone()]);
        for field in &mut self.fields {
            field.ftype = type_scope.resolve(&field.ftype)?;
            type_scope.check_for_recursion(&field.ftype, field.span, &mut parent_structs)?;
        }
        type_scope.pop();
        type_scope.insert(self.name.clone(), Type::Struct(Box::new(self.clone())));
        Ok(())
    }
}

impl IfStatement {
    pub fn check_types(&mut self, type_scope: &mut TypeScope) -> Result<(), TypeCheckError> {
        check_types_expression(&mut self.condition, type_scope)?;
        self.block.check_types(type_scope)?;
        if let Some(else_block) = &mut self.else_block {
            else_block.check_types(type_scope)?;
        }
        Ok(())
    }

    pub fn resolve_types(&mut self, type_scope: &TypeScope) {
        self.block.resolve_types(type_scope);
        if let Some(else_block) = &mut self.else_block {
            else_block.resolve_types(type_scope);
        }
    }
}

impl MatchStatement {
    pub fn check_types(&mut self, type_scope: &mut TypeScope) -> Result<(), TypeCheckError> {
        let expected = check_types_expression(&mut self.lhs, type_scope)?;
        for case in &mut self.cases {
            let got = check_types_expression(&mut case.rhs, type_scope)?;
            expect_type(&case.rhs, &expected, &got, type_scope)?;
            case.block.check_types(type_scope)?;
        }
        if let Some(default) = &mut self.default {
            return default.check_types(type_scope);
        }
        self.check_exhaustive(&expected)
    }

    fn check_exhaustive(&self, matched: &Type) -> Result<(), TypeCheckError> {
        let Type::Enum(enum_def) = matched else {
            return Ok(());
        };

        let covered: HashSet<&str> = self
            .cases
            .iter()
            .filter_map(|case| match &case.rhs.kind {
                ExpressionKind::EnumVariant { variant_name, .. } => Some(variant_name.as_str()),
                _ => None,
            })
            .collect();

        let missing: Vec<String> = enum_def
            .variants
            .iter()
            .filter(|variant| !covered.contains(variant.as_str()))
            .cloned()
            .collect();

        if missing.is_empty() {
            Ok(())
        } else {
            Err(TypeCheckErrorKind::NonExhaustiveMatch {
                enum_type: enum_def.clone(),
                missing,
            }
            .at(self.span))
        }
    }

    pub fn resolve_types(&mut self, type_scope: &TypeScope) {
        for case in &mut self.cases {
            case.block.resolve_types(type_scope);
        }
        if let Some(default) = &mut self.default {
            default.resolve_types(type_scope);
        }
    }
}

impl ReturnStatement {
    pub fn check_types(&mut self, type_scope: &mut TypeScope) -> Result<(), TypeCheckError> {
        let got = match &mut self.expr {
            Some(expr) => check_types_expression(expr, type_scope)?,
            None => Type::Unit,
        };
        let Some(expected) = type_scope.expected_return_type().cloned() else {
            return Ok(());
        };
        match &self.expr {
            Some(expr) => expect_type(expr, &expected, &got, type_scope),
            None if expected == Type::Unit => Ok(()),
            None => Err(TypeCheckErrorKind::TypeMismatch {
                expected: Box::new(expected),
                got: Box::new(got),
            }
            .at(self.span)),
        }
    }
}

impl WhileLoop {
    pub fn check_types(&mut self, type_scope: &mut TypeScope) -> Result<(), TypeCheckError> {
        check_types_expression(&mut self.condition, type_scope)?;
        self.block.check_types(type_scope)
    }
}
