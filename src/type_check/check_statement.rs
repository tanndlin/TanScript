use crate::{
    ast::{
        Assignment, AtomType, Block, Declaration, ExpressionKind, ForLoop, FunctionDefinition,
        IfStatement, ReturnStatement, Statement, StatementOrExpression, WhileLoop,
    },
    error::At,
    type_check::{
        FunctionType, Type, TypeCheckError, TypeCheckErrorKind, TypeScope, check_types_expression,
        expect_type, narrow_expression,
    },
    types::IntType,
};

impl Block {
    pub fn check_types(&mut self, type_scope: &mut TypeScope) -> Result<(), TypeCheckError> {
        type_scope.push();
        let result = self
            .children
            .iter_mut()
            .try_for_each(|child| child.check_types(type_scope));
        type_scope.pop();
        result
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
            Self::IfStatement(s) => s.check_types(type_scope),
            Self::Return(s) => s.check_types(type_scope),
            Self::WhileLoop(s) => s.check_types(type_scope),
            Self::Break(_) | Self::Continue(_) | Self::StructDefinition(_) => Ok(()),
        }
    }

    pub fn resolve_types(&mut self, type_scope: &TypeScope) {
        match self {
            Self::Declaration(s) => s.resolve_types(type_scope),
            Self::ForLoop(s) => s.resolve_types(type_scope),
            Self::FunctionDefinition(s) => s.body.resolve_types(type_scope),
            Self::IfStatement(s) => s.resolve_types(type_scope),
            Self::WhileLoop(s) => s.block.resolve_types(type_scope),
            Self::Assignment(_)
            | Self::Return(_)
            | Self::Break(_)
            | Self::Continue(_)
            | Self::StructDefinition(_) => {}
        }
    }
}

impl Assignment {
    pub fn check_types(&mut self, type_scope: &mut TypeScope) -> Result<(), TypeCheckError> {
        let expected = check_types_expression(&self.lhs, type_scope)?;
        let got = check_types_expression(&self.expression, type_scope)?;

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

        let got = check_types_expression(&self.assign.expression, type_scope)?;
        let Some(expected) = self.dtype.clone() else {
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
            check_types_expression(&self.condition, type_scope)?;
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
        type_scope.insert(
            self.name.clone(),
            Type::Function(Box::new(FunctionType {
                args: self.args.iter().map(|a| a.atype.clone()).collect(),
                return_type: self.return_type.clone(),
            })),
        );

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
                expected: self.return_type.clone(),
                got: Type::Unit,
            }
            .at(self.span));
        }
        Ok(())
    }
}

impl IfStatement {
    pub fn check_types(&mut self, type_scope: &mut TypeScope) -> Result<(), TypeCheckError> {
        check_types_expression(&self.condition, type_scope)?;
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

impl ReturnStatement {
    pub fn check_types(&mut self, type_scope: &mut TypeScope) -> Result<(), TypeCheckError> {
        let got = match &self.expr {
            Some(expr) => check_types_expression(expr, type_scope)?,
            None => Type::Unit,
        };
        let Some(expected) = type_scope.expected_return_type().cloned() else {
            return Ok(());
        };
        match &self.expr {
            Some(expr) => expect_type(expr, &expected, &got, type_scope),
            None if expected == Type::Unit => Ok(()),
            None => Err(TypeCheckErrorKind::TypeMismatch { expected, got }.at(self.span)),
        }
    }
}

impl WhileLoop {
    pub fn check_types(&mut self, type_scope: &mut TypeScope) -> Result<(), TypeCheckError> {
        check_types_expression(&self.condition, type_scope)?;
        self.block.check_types(type_scope)
    }
}
