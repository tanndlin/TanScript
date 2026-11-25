use std::{borrow::Cow, collections::HashSet, rc::Rc};

use crate::{
    ast::{
        AssignOp, Assignment, Block, Declaration, EnumDefinition, Expression, ExpressionKind,
        ExternDeclaration, ForLoop, FunctionDefinition, IfStatement, MatchStatement,
        ReturnStatement, Signature, Statement, StatementOrExpression, StructDefinition, UnaryOp,
        WhileLoop,
    },
    error::At,
    type_check::{
        FunctionType, Type, TypeCheckError, TypeCheckErrorKind, TypeScope, check_binary,
        check_types_expression, check_types_expression_expecting, expect_type, narrow_expression,
        unify,
    },
    types::IntType,
};

impl Block {
    pub fn check_types(&mut self, type_scope: &mut TypeScope) -> Result<(), TypeCheckError> {
        type_scope.push();
        let result = self.declare(type_scope).and_then(|()| {
            self.children.iter_mut().try_for_each(|child| match child {
                StatementOrExpression::Statement(statement) => match statement.as_mut() {
                    Statement::StructDefinition(_)
                    | Statement::EnumDefinition(_)
                    | Statement::ExternDeclaration(_) => Ok(()),
                    Statement::FunctionDefinition(s) => check_function_body(s, type_scope),
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
            check_struct(structs[i], type_scope)?;
        }

        for child in &mut self.children {
            if let StatementOrExpression::Statement(statement) = child {
                match statement.as_mut() {
                    Statement::FunctionDefinition(s) => declare_function(s, type_scope)?,
                    Statement::ExternDeclaration(s) => declare_extern_function(s, type_scope)?,
                    _ => {}
                }
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

fn check_struct(
    def: &mut Rc<StructDefinition>,
    type_scope: &mut TypeScope,
) -> Result<(), TypeCheckError> {
    Rc::make_mut(def).check_types(type_scope)?;
    type_scope.insert(def.name.clone(), Type::Struct(Rc::clone(def)));
    Ok(())
}

fn value_dependency_order(structs: &[&mut Rc<StructDefinition>]) -> Vec<usize> {
    let mut visited = vec![false; structs.len()];
    let mut order = vec![];
    for i in 0..structs.len() {
        visit_value_dependencies(i, structs, &mut visited, &mut order);
    }
    order
}

fn visit_value_dependencies(
    i: usize,
    structs: &[&mut Rc<StructDefinition>],
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
        match self {
            StatementOrExpression::Statement(statement) => statement.resolve_types(type_scope),
            StatementOrExpression::Expression(expression) => expression.resolve_types(type_scope),
        }
    }
}

impl Statement {
    pub fn check_types(&mut self, type_scope: &mut TypeScope) -> Result<(), TypeCheckError> {
        match self {
            Self::Assignment(s) => s.check_types(type_scope),
            Self::Declaration(s) => s.check_types(type_scope),
            Self::ForLoop(s) => s.check_types(type_scope),
            Self::FunctionDefinition(s) => {
                declare_function(s, type_scope)?;
                check_function_body(s, type_scope)
            }
            Self::ExternDeclaration(s) => declare_extern_function(s, type_scope),
            Self::StructDefinition(s) => {
                s.declare(type_scope)?;
                check_struct(s, type_scope)
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
            Self::FunctionDefinition(s) if s.signature.generics.is_empty() => {
                Rc::make_mut(s).body.resolve_types(type_scope);
            }
            Self::IfStatement(s) => s.resolve_types(type_scope),
            Self::MatchStatement(s) => s.resolve_types(type_scope),
            Self::WhileLoop(s) => s.resolve_types(type_scope),
            Self::Return(s) => s.resolve_types(type_scope),
            Self::Assignment(s) => s.resolve_types(type_scope),
            Self::Break(_)
            | Self::Continue(_)
            | Self::FunctionDefinition(_)
            | Self::StructDefinition(_)
            | Self::ExternDeclaration(_)
            | Self::EnumDefinition(_) => {}
        }
    }
}

impl Assignment {
    pub fn check_types(&mut self, type_scope: &mut TypeScope) -> Result<(), TypeCheckError> {
        let expected = check_types_expression(&mut self.lhs, type_scope)?.clone();
        let got = match self.op {
            AssignOp::Assign => check_types_expression(&mut self.expression, type_scope)?.clone(),
            AssignOp::Compound(op) => {
                check_binary(op, &mut self.lhs, &mut self.expression, type_scope)?
            }
        };

        if let (Type::Int(e), Type::Int(g)) = (&expected, &got)
            && e.coerces_to(*g)
        {
            narrow_expression(&self.lhs, &got, type_scope);
            return Ok(());
        }
        expect_type(&self.expression, &expected, &got, type_scope)
    }

    pub fn resolve_types(&mut self, type_scope: &TypeScope) {
        self.lhs.resolve_types(type_scope);
        self.expression.resolve_types(type_scope);
    }
}

impl Declaration {
    pub fn check_types(&mut self, type_scope: &mut TypeScope) -> Result<(), TypeCheckError> {
        let expected = self
            .annotation
            .as_ref()
            .map(|t| type_scope.resolve(t).map(Cow::into_owned))
            .transpose()?;
        let got = check_types_expression_expecting(&mut self.value, expected.as_ref(), type_scope)?
            .clone();
        let declared = match expected {
            Some(expected) => {
                expect_type(&self.value, &expected, &got, type_scope)?;
                expected
            }
            None => got,
        };

        self.binding = Some(type_scope.insert(self.ident.clone(), declared));
        Ok(())
    }

    pub fn resolve_types(&mut self, type_scope: &TypeScope) {
        if let Some(id) = self.binding {
            self.ident_type = Some(type_scope.binding(id).clone());
        }
        self.value.resolve_types(type_scope);
    }
}

impl ForLoop {
    pub fn check_types(&mut self, type_scope: &mut TypeScope) -> Result<(), TypeCheckError> {
        let start = check_types_expression(&mut self.range.start, type_scope)?.clone();
        let end = check_types_expression(&mut self.range.end, type_scope)?.clone();

        for (bound, span) in [(&start, self.range.start.span), (&end, self.range.end.span)] {
            if *bound != Type::Unknown && !bound.is_number_type() {
                return Err(
                    TypeCheckErrorKind::mismatch(Type::Int(IntType::I64), bound.clone()).at(span),
                );
            }
        }

        let unified = unify(&start, &end)
            .or_else(|| unify(&end, &start))
            .cloned()
            .ok_or_else(|| {
                TypeCheckErrorKind::mismatch(start.clone(), end.clone()).at(self.range.end.span)
            })?;

        type_scope.push();
        self.binding = Some(type_scope.insert(self.ident.clone(), start));
        type_scope.narrow(&self.ident, unified);
        let result = self.block.check_types(type_scope);
        type_scope.pop();
        result
    }

    pub fn resolve_types(&mut self, type_scope: &TypeScope) {
        if let Some(id) = self.binding {
            self.ident_type = Some(type_scope.binding(id).clone());
        }
        self.block.resolve_types(type_scope);
        self.range.start.resolve_types(type_scope);
        self.range.end.resolve_types(type_scope);
    }
}

fn declare_function(
    def: &mut Rc<FunctionDefinition>,
    type_scope: &mut TypeScope,
) -> Result<(), TypeCheckError> {
    if !def.signature.generics.is_empty() {
        type_scope.declare_generic_function(Rc::clone(def));
        return Ok(());
    }

    Rc::make_mut(def).signature.declare(type_scope)
}

fn check_function_body(
    def: &mut Rc<FunctionDefinition>,
    type_scope: &mut TypeScope,
) -> Result<(), TypeCheckError> {
    if !def.signature.generics.is_empty() {
        return Ok(());
    }

    Rc::make_mut(def).check_body(type_scope)
}

fn declare_extern_function(
    def: &mut Rc<ExternDeclaration>,
    type_scope: &mut TypeScope,
) -> Result<(), TypeCheckError> {
    if !def.signature.generics.is_empty() {
        type_scope.declare_generic_extern(Rc::clone(def));
        return Ok(());
    }

    Rc::make_mut(def).signature.declare(type_scope)
}

impl Signature {
    pub fn function_type(&self) -> FunctionType {
        FunctionType {
            args: self.args.iter().map(|a| a.atype.clone()).collect(),
            return_type: self.return_type.clone(),
        }
    }

    fn declare(&mut self, type_scope: &mut TypeScope) -> Result<(), TypeCheckError> {
        for arg in &mut self.args {
            type_scope.resolve_in_place(&mut arg.atype)?;
        }
        type_scope.resolve_in_place(&mut self.return_type)?;
        type_scope.insert(
            self.name.clone(),
            Type::Function(Box::new(self.function_type())),
        );
        Ok(())
    }
}

impl FunctionDefinition {
    pub fn check_body(&mut self, type_scope: &mut TypeScope) -> Result<(), TypeCheckError> {
        type_scope.push();
        type_scope.push_function(self.signature.return_type.clone());
        for arg in &self.signature.args {
            type_scope.insert(arg.name.clone(), arg.atype.clone());
        }
        let result = self.body.check_types(type_scope);
        type_scope.pop_function();
        type_scope.pop();
        result?;

        let signature = &self.signature;
        if signature.return_type != Type::Unit && !self.body.always_returns() {
            return Err(
                TypeCheckErrorKind::mismatch(signature.return_type.clone(), Type::Unit)
                    .at(signature.span),
            );
        }
        Ok(())
    }
}

impl EnumDefinition {
    pub fn check_types(self: &Rc<Self>, type_scope: &mut TypeScope) -> Result<(), TypeCheckError> {
        if type_scope.lookup(&self.name).is_some() {
            return Err(TypeCheckErrorKind::DuplicateIdentifiers(self.name.clone()).at(self.span));
        }

        for (i, variant) in self.variants.iter().enumerate() {
            if self.variants[..i].contains(variant) {
                return Err(TypeCheckErrorKind::DuplicateIdentifiers(variant.clone()).at(self.span));
            }
        }

        type_scope.insert(self.name.clone(), Type::Enum(Rc::clone(self)));
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
        for generic_name in &self.generics {
            type_scope.insert(generic_name.clone(), Type::Generic(generic_name.clone()));
        }

        let mut parent_structs = HashSet::from([self.name.clone()]);
        for field in &mut self.fields {
            type_scope.resolve_in_place(&mut field.ftype)?;
            type_scope.check_for_recursion(&field.ftype, field.span, &mut parent_structs)?;
        }
        type_scope.pop();
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
        self.condition.resolve_types(type_scope);
        self.block.resolve_types(type_scope);
        if let Some(else_block) = &mut self.else_block {
            else_block.resolve_types(type_scope);
        }
    }
}

impl MatchStatement {
    pub fn check_types(&mut self, type_scope: &mut TypeScope) -> Result<(), TypeCheckError> {
        let expected = check_types_expression(&mut self.lhs, type_scope)?.clone();
        for case in &mut self.cases {
            let got = check_types_expression(&mut case.rhs, type_scope)?.clone();
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
        self.lhs.resolve_types(type_scope);
        for case in &mut self.cases {
            case.rhs.resolve_types(type_scope);
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
            Some(expr) => check_types_expression(expr, type_scope)?.clone(),
            None => Type::Unit,
        };
        let Some(expected) = type_scope.expected_return_type().cloned() else {
            return Ok(());
        };
        match &self.expr {
            Some(expr) => expect_type(expr, &expected, &got, type_scope),
            None if expected == Type::Unit => Ok(()),
            None => Err(TypeCheckErrorKind::mismatch(expected, got).at(self.span)),
        }
    }

    pub fn resolve_types(&mut self, type_scope: &TypeScope) {
        if let Some(expr) = &mut self.expr {
            expr.resolve_types(type_scope);
        }
    }
}

impl WhileLoop {
    pub fn check_types(&mut self, type_scope: &mut TypeScope) -> Result<(), TypeCheckError> {
        check_types_expression(&mut self.condition, type_scope)?;
        self.block.check_types(type_scope)
    }

    pub fn resolve_types(&mut self, type_scope: &TypeScope) {
        self.condition.resolve_types(type_scope);
        self.block.resolve_types(type_scope);
    }
}

impl Expression {
    pub fn resolve_types(&mut self, type_scope: &TypeScope) {
        let resolved = match &mut self.kind {
            ExpressionKind::Variable {
                binding: Some(id), ..
            } => Some(type_scope.binding(*id).clone()),
            ExpressionKind::Binary(op, lhs, rhs) => {
                lhs.resolve_types(type_scope);
                rhs.resolve_types(type_scope);
                if op.returns_boolean() {
                    None
                } else {
                    lhs.try_etype()
                        .zip(rhs.try_etype())
                        .and_then(|(l, r)| unify(l, r).or_else(|| unify(r, l)))
                        .cloned()
                }
            }
            ExpressionKind::Unary(op, inner) => {
                inner.resolve_types(type_scope);
                match (*op, inner.try_etype()) {
                    (UnaryOp::Negate, Some(Type::Int(t))) => Some(Type::Int(t.negated())),
                    (UnaryOp::AddressOf, Some(t)) => Some(t.clone().pointer_to()),
                    (UnaryOp::Deref, Some(Type::Pointer(pointee)))
                        if !matches!(**pointee, Type::Named { .. }) =>
                    {
                        Some(Type::clone(pointee))
                    }
                    _ => None,
                }
            }
            ExpressionKind::Postfix(_, inner) => {
                inner.resolve_types(type_scope);
                inner.try_etype().cloned()
            }
            ExpressionKind::Index(base, index) => {
                base.resolve_types(type_scope);
                index.resolve_types(type_scope);
                match base.try_etype() {
                    Some(Type::Pointer(element)) if !matches!(**element, Type::Named { .. }) => {
                        Some(Type::clone(element))
                    }
                    _ => None,
                }
            }
            ExpressionKind::Cast(inner, _) | ExpressionKind::StructAccess { base: inner, .. } => {
                inner.resolve_types(type_scope);
                None
            }
            ExpressionKind::FunctionCall(_, args) => {
                for arg in args {
                    arg.resolve_types(type_scope);
                }
                None
            }
            ExpressionKind::StructInstantiation { fields, .. } => {
                for (_, field) in fields {
                    field.resolve_types(type_scope);
                }
                None
            }
            ExpressionKind::Variable { binding: None, .. }
            | ExpressionKind::Atom(_)
            | ExpressionKind::SizeOf(_)
            | ExpressionKind::EnumVariant { .. } => None,
        };
        if let Some(t) = resolved {
            self.set_etype(t);
        }
    }
}
