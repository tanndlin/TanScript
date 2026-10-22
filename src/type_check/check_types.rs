use crate::{
    ast::{AtomType, BinaryOp, Expression, ExpressionKind, Program, UnaryOp},
    error::At,
    type_check::{Type, TypeCheckError, TypeCheckErrorKind, TypeScope, infer_type::unify},
    types::{IntType, Span},
};

pub fn check_types(program: &mut Program) -> Result<(), TypeCheckError> {
    let mut type_scope = TypeScope::default();

    program.block.check_types(&mut type_scope)?;
    program.block.resolve_types(&type_scope);
    Ok(())
}

pub fn check_types_expression(
    expression: &mut Expression,
    type_scope: &mut TypeScope,
) -> Result<Type, TypeCheckError> {
    let span = expression.span;
    let t = match &mut expression.kind {
        ExpressionKind::Atom(AtomType::Identifier(ident)) => type_scope
            .lookup(ident)
            .cloned()
            .ok_or_else(|| TypeCheckErrorKind::IdentifierNotFound(ident.clone()).at(span)),
        ExpressionKind::Atom(atom_type) => Ok(Type::from(&*atom_type)),
        ExpressionKind::Binary(op, lhs, rhs) => check_binary(*op, lhs, rhs, type_scope),
        ExpressionKind::Unary(UnaryOp::Not, inner) => {
            check_types_expression(inner, type_scope)?;
            Ok(Type::Boolean)
        }
        ExpressionKind::Unary(UnaryOp::AddressOf, inner) => Ok(Type::Pointer(Box::new(
            check_types_expression(inner, type_scope)?,
        ))),
        ExpressionKind::Unary(UnaryOp::Deref, inner) => {
            match check_types_expression(inner, type_scope)? {
                Type::Pointer(pointee) => Ok(type_scope.expand(*pointee)),
                other => Err(TypeCheckErrorKind::NotDerefable(other).at(inner.span)),
            }
        }
        ExpressionKind::Unary(_, inner) | ExpressionKind::Postfix(_, inner) => {
            check_types_expression(inner, type_scope)
        }
        ExpressionKind::FunctionCall(function_name, given_args) => {
            check_function_call(span, function_name, given_args, type_scope)
        }
        ExpressionKind::Index(lhs, rhs) => check_index(span, lhs, rhs, type_scope),
        ExpressionKind::Cast(lhs, casted_type) => check_cast(span, lhs, casted_type, type_scope),
        ExpressionKind::StructInstantiation {
            struct_type_name,
            fields,
        } => check_struct_instantiation(span, struct_type_name, fields, type_scope),
        ExpressionKind::StructAccess { base, field_name } => {
            check_struct_access(span, base, field_name, type_scope)
        }
        ExpressionKind::EnumVariant {
            enum_name,
            variant_name,
        } => check_enum_variant(span, enum_name, variant_name, type_scope),
    }?;
    *expression.etype = t.clone();
    Ok(t)
}

pub fn check_enum_variant(
    span: Span,
    enum_name: &str,
    variant_name: &str,
    type_scope: &TypeScope,
) -> Result<Type, TypeCheckError> {
    let Some(Type::Enum(enum_def)) = type_scope.lookup(enum_name) else {
        return Err(TypeCheckErrorKind::NotAnEnum(enum_name.to_string()).at(span));
    };

    if enum_def.variant_index(variant_name).is_none() {
        return Err(TypeCheckErrorKind::NoSuchVariant {
            enum_type: enum_def.clone(),
            variant_name: variant_name.to_string(),
        }
        .at(span));
    }

    Ok(Type::Enum(enum_def.clone()))
}

pub fn check_binary(
    op: BinaryOp,
    lhs: &mut Expression,
    rhs: &mut Expression,
    type_scope: &mut TypeScope,
) -> Result<Type, TypeCheckError> {
    let lhs_type = check_types_expression(lhs, type_scope)?;
    let rhs_type = check_types_expression(rhs, type_scope)?;

    // Cant do a binary op on a struct
    if let Some((struct_type, operand)) = [(&lhs_type, &*lhs), (&rhs_type, &*rhs)]
        .into_iter()
        .find(|(t, _)| matches!(t, Type::Struct(_)))
    {
        return Err(TypeCheckErrorKind::TypeMismatch {
            expected: Box::new(Type::Int(IntType::I64)),
            got: Box::new(struct_type.clone()),
        }
        .at(operand.span));
    }

    let unified = unify(&lhs_type, &rhs_type)
        .or_else(|| unify(&rhs_type, &lhs_type))
        .ok_or_else(|| {
            TypeCheckErrorKind::TypeMismatch {
                expected: Box::new(lhs_type.clone()),
                got: Box::new(rhs_type.clone()),
            }
            .at(rhs.span)
        })?;

    Ok(if op.returns_boolean() {
        Type::Boolean
    } else {
        unified
    })
}

pub fn check_function_call(
    span: Span,
    function_name: &str,
    given_args: &mut [Expression],
    type_scope: &mut TypeScope,
) -> Result<Type, TypeCheckError> {
    let given_types = given_args
        .iter_mut()
        .map(|a| check_types_expression(a, type_scope))
        .collect::<Result<Vec<_>, _>>()?;

    match type_scope.lookup(function_name) {
        Some(Type::Function(function_type)) => {
            if given_args.len() != function_type.args.len() {
                return Err(TypeCheckErrorKind::IncorrectNumberOfArgs {
                    expected: function_type.args.len(),
                    given: given_args.len(),
                }
                .at(span));
            }

            let function_type = function_type.clone();
            given_args
                .iter()
                .zip(given_types)
                .zip(&function_type.args)
                .try_for_each(|((given, got), expected)| {
                    expect_type(given, expected, &got, type_scope)
                })?;
            Ok(function_type.return_type)
        }
        Some(_) => {
            Err(TypeCheckErrorKind::NotCallableIdentifer(function_name.to_string()).at(span))
        }
        // Treat the function as an unknown extern
        None => match given_args
            .iter()
            .zip(&given_types)
            .find(|(_, t)| matches!(t, Type::Struct(_)))
        {
            Some((given, _)) => {
                Err(TypeCheckErrorKind::StructAsArgument(function_name.to_string()).at(given.span))
            }
            None => Ok(Type::Unknown),
        },
    }
}

pub fn check_index(
    span: Span,
    lhs: &mut Expression,
    rhs: &mut Expression,
    type_scope: &mut TypeScope,
) -> Result<Type, TypeCheckError> {
    let lhs_type = check_types_expression(lhs, type_scope)?;
    let rhs_type = check_types_expression(rhs, type_scope)?;

    // LHS must be a pointer
    let element_type = match lhs_type {
        Type::Pointer(t) => type_scope.expand(*t),
        Type::Unknown => return Err(TypeCheckErrorKind::NotIndexable(lhs_type).at(lhs.span)),
        _ => {
            return Err(TypeCheckErrorKind::TypeMismatch {
                expected: Box::new(Type::Pointer(Box::new(Type::Unknown))),
                got: Box::new(lhs_type),
            }
            .at(span));
        }
    };

    // RHS must come down to a number
    if rhs_type != Type::Unknown && !rhs_type.is_number_type() {
        return Err(TypeCheckErrorKind::TypeMismatch {
            expected: Box::new(Type::Int(IntType::I64)),
            got: Box::new(rhs_type),
        }
        .at(span));
    }

    Ok(element_type)
}

pub fn check_cast(
    span: Span,
    lhs: &mut Expression,
    casted_type: &mut Type,
    type_scope: &mut TypeScope,
) -> Result<Type, TypeCheckError> {
    let lhs_type = check_types_expression(lhs, type_scope)?;
    *casted_type = type_scope.resolve(casted_type)?;
    if lhs_type.castable(casted_type) {
        Ok(casted_type.clone())
    } else {
        Err(TypeCheckErrorKind::Uncastable {
            lhs_type: Box::new(lhs_type),
            casted_type: Box::new(casted_type.clone()),
        }
        .at(span))
    }
}

pub fn check_struct_instantiation(
    span: Span,
    struct_type_name: &str,
    fields: &mut [(String, Expression)],
    type_scope: &mut TypeScope,
) -> Result<Type, TypeCheckError> {
    match type_scope.lookup(struct_type_name) {
        Some(Type::Struct(struct_def)) => {
            let struct_def = struct_def.clone();
            for (field_name, field_expr) in fields.iter_mut() {
                let Some(field) = struct_def.fields.iter().find(|f| f.name == *field_name) else {
                    return Err(TypeCheckErrorKind::NoSuchField {
                        struct_type: struct_def.clone(),
                        field_name: field_name.clone(),
                    }
                    .at(field_expr.span));
                };

                let got = check_types_expression(field_expr, type_scope)?;
                expect_type(field_expr, &field.ftype, &got, type_scope)?;
            }

            if let Some(missing) = struct_def
                .fields
                .iter()
                .find(|f| !fields.iter().any(|(name, _)| *name == f.name))
            {
                return Err(TypeCheckErrorKind::MissingField {
                    struct_type: struct_def.clone(),
                    field_name: missing.name.clone(),
                }
                .at(span));
            }

            Ok(Type::Struct(struct_def))
        }
        Some(_) => Err(TypeCheckErrorKind::NotAStruct(struct_type_name.to_string()).at(span)),
        None => Err(TypeCheckErrorKind::IdentifierNotFound(struct_type_name.to_string()).at(span)),
    }
}

pub fn check_struct_access(
    span: Span,
    base: &mut Expression,
    field_name: &str,
    type_scope: &mut TypeScope,
) -> Result<Type, TypeCheckError> {
    match check_types_expression(base, type_scope)? {
        Type::Struct(struct_type) => struct_type
            .fields
            .iter()
            .find(|f| f.name == field_name)
            .map(|field| field.ftype.clone())
            .ok_or_else(|| {
                TypeCheckErrorKind::NoSuchField {
                    struct_type: struct_type.clone(),
                    field_name: field_name.to_string(),
                }
                .at(span)
            }),
        t => Err(TypeCheckErrorKind::NotAStruct(t.to_string()).at(span)),
    }
}

pub fn expect_type(
    expression: &Expression,
    expected: &Type,
    got: &Type,
    type_scope: &mut TypeScope,
) -> Result<(), TypeCheckError> {
    match (expected, got) {
        _ if expected == got => Ok(()),
        (Type::Unknown, _) | (_, Type::Unknown) | (Type::Pointer(_), Type::Null) => Ok(()),
        (Type::Int(to), Type::Int(from)) if from.widens_to(*to) => Ok(()),
        (t, Type::Int(IntType::Coercible)) if t.is_number_type() => {
            narrow_expression(expression, expected.clone(), type_scope);
            Ok(())
        }
        (Type::Pointer(to), Type::Pointer(from))
            if to.is_number_type() && **from == Type::Int(IntType::Coercible) =>
        {
            narrow_expression(expression, expected.clone(), type_scope);
            Ok(())
        }
        _ => Err(TypeCheckErrorKind::TypeMismatch {
            expected: Box::new(expected.clone()),
            got: Box::new(got.clone()),
        }
        .at(expression.span)),
    }
}

pub fn narrow_expression(expression: &Expression, t: Type, type_scope: &mut TypeScope) {
    match &expression.kind {
        ExpressionKind::Atom(AtomType::Identifier(ident)) => type_scope.narrow(ident, t),
        ExpressionKind::Binary(_, lhs, rhs) => {
            narrow_expression(lhs, t.clone(), type_scope);
            narrow_expression(rhs, t, type_scope);
        }
        ExpressionKind::Unary(UnaryOp::AddressOf, inner) => {
            if let Type::Pointer(pointee) = t {
                narrow_expression(inner, *pointee, type_scope);
            }
        }
        ExpressionKind::Unary(_, inner) | ExpressionKind::Postfix(_, inner) => {
            narrow_expression(inner, t, type_scope);
        }
        _ => {}
    }
}

#[cfg(test)]
mod test {
    use crate::{
        ast::{Program, Statement, StatementOrExpression},
        lex::lex,
        parse::parse,
        type_check::{Type, TypeCheckError, TypeCheckErrorKind, check_types},
        types::IntType,
    };

    const POINT: &str = "struct P { x: u32, y: bool }";

    fn check(source: &str) -> Result<(), TypeCheckError> {
        let mut program = parse(lex(source, 0).unwrap()).unwrap();
        check_types(&mut program)
    }

    fn checked(source: &str) -> Program {
        let mut program = parse(lex(source, 0).unwrap()).unwrap();
        check_types(&mut program).unwrap();
        program
    }

    macro_rules! expect_err {
        ($source:expr, $kind:pat $(if $guard:expr)?) => {{
            let source = $source;
            let result = check(&source);
            assert!(
                matches!(result, Err(TypeCheckError { kind: $kind, .. }) $(if $guard)?),
                "expected {} for {source}, got {result:?}",
                stringify!($kind)
            );
        }};
    }

    macro_rules! expect_mismatch {
        ($source:expr) => {
            expect_err!($source, TypeCheckErrorKind::TypeMismatch { .. })
        };
    }

    #[test]
    fn supports_coercing_literal_assignments() -> Result<(), TypeCheckError> {
        check("let a: u32 = 1;")
    }

    #[test]
    fn supports_upcasting_u8_to_u32() -> Result<(), TypeCheckError> {
        check("let a: u8 = 1; let b: u32 = a;")
    }

    #[test]
    fn supports_upcasting_u8_to_u64() -> Result<(), TypeCheckError> {
        check("let a: u8 = 1; let b: u64 = a;")
    }

    #[test]
    fn supports_upcasting_u32_to_u64() -> Result<(), TypeCheckError> {
        check("let a: u32 = 1; let b: u64 = a;")
    }

    #[test]
    fn supports_upcasting_in_assignment() -> Result<(), TypeCheckError> {
        check("let a: u8 = 1; let b: u64 = 2; b = a;")
    }

    #[test]
    fn supports_upcasting_function_arg() -> Result<(), TypeCheckError> {
        check("def f (x: u64) {} let a: u8 = 1; f(a);")
    }

    #[test]
    fn supports_upcasting_return() -> Result<(), TypeCheckError> {
        check("def f (a: u8): u32 { a }")
    }

    #[test]
    fn type_error_for_downcasting_u64_to_u32() {
        expect_mismatch!("let a: u64 = 1; let b: u32 = a;");
    }

    #[test]
    fn supports_upcasting_u32_to_i64() -> Result<(), TypeCheckError> {
        check("let a: u32 = 1; let b: i64 = a;")
    }

    #[test]
    fn supports_upcasting_u8_to_i32() -> Result<(), TypeCheckError> {
        check("let a: u8 = 1; let b: i32 = a;")
    }

    #[test]
    fn type_error_for_u32_to_i32() {
        expect_mismatch!("let a: u32 = 1; let b: i32 = a;");
    }

    #[test]
    fn type_error_for_i32_to_u64() {
        expect_mismatch!("let a: i32 = 1; let b: u64 = a;");
    }

    #[test]
    fn type_error_for_incorrect_function_arg() {
        expect_mismatch!(
            "def add (a: u32, b: u32): u32 {a + b} let a: i32 = 1; let b: u32 = 2; add(a,b);"
        );
    }

    #[test]
    fn type_error_for_coerced_variable_used_as_arg_then_assigned_to_different_type() {
        expect_mismatch!(
            "def add (a: u32, b: u32): u32 {a + b} let a = 6; let b = 9; add(a, b); let c: i32 = a;"
        );
    }

    #[test]
    fn type_error_for_coerced_variable_assigned_then_used_as_arg_of_different_type() {
        expect_mismatch!(
            "def add (a: u32, b: u32): u32 {a + b} let a = 6; let b = 9; let c: i32 = a; add(a, b);"
        );
    }

    #[test]
    fn supports_boolean_declaration() -> Result<(), TypeCheckError> {
        check("let a: bool = true; let b: bool = false; let c: bool = a;")
    }

    #[test]
    fn type_error_for_boolean_assigned_to_number() {
        expect_mismatch!("let a: u32 = true;");
    }

    #[test]
    fn type_error_for_number_assigned_to_boolean() {
        expect_mismatch!("let a: bool = 1;");
    }

    #[test]
    fn type_error_for_boolean_function_arg() {
        expect_mismatch!("def f (a: u32): u32 {a} f(true);");
    }

    #[test]
    fn supports_comparison_assigned_to_boolean() -> Result<(), TypeCheckError> {
        check("let a: i32 = 1; let b: bool = a < 2;")
    }

    #[test]
    fn supports_using_function_params_in_body() -> Result<(), TypeCheckError> {
        check("def f (arr: *i32, i: i32): i32 { let x = arr[i]; x }")
    }

    #[test]
    fn supports_untyped_char_variable_as_char_arg() -> Result<(), TypeCheckError> {
        check("def f(c: char) {} let c = 'c'; f(c);")
    }

    #[test]
    fn untyped_char_declaration_is_char() {
        let program = checked("let c = 'c';");

        let StatementOrExpression::Statement(statement) = &program.block.children[0] else {
            panic!("expected a statement");
        };
        let Statement::Declaration(declaration) = statement.as_ref() else {
            panic!("expected a declaration");
        };
        assert_eq!(declaration.dtype, Some(Type::Char));
    }

    #[test]
    fn shadowed_declaration_keeps_its_own_type() {
        let program =
            checked("let i: bool = true; for (let i = 0; i < 10; i++) { let j: i32 = i; }");

        let StatementOrExpression::Statement(statement) = &program.block.children[1] else {
            panic!("expected a statement");
        };
        let Statement::ForLoop(for_loop) = statement.as_ref() else {
            panic!("expected a for loop");
        };
        let StatementOrExpression::Statement(init) = &for_loop.init else {
            panic!("expected a statement");
        };
        let Statement::Declaration(declaration) = init.as_ref() else {
            panic!("expected a declaration");
        };
        assert_eq!(declaration.dtype, Some(Type::Int(IntType::I32)));
    }

    #[test]
    fn type_error_inside_loop_body() {
        expect_mismatch!("let a: u32 = 1; while true { let b: i32 = a; }");
    }

    #[test]
    fn cast_allows_assignment_of_different_type() -> Result<(), TypeCheckError> {
        check("let a: u32 = 1; let b: i32 = 2; a = b as u32;")
    }

    #[test]
    fn type_error_for_uncastable_types() {
        expect_err!(
            "let a: i64 = 1; let b: bool = a as bool;",
            TypeCheckErrorKind::Uncastable { .. }
        );
    }

    #[test]
    fn type_error_for_mismatched_assignment() {
        expect_mismatch!("let a: u32 = 1; let b: i32 = 2; a = b;");
    }

    #[test]
    fn supports_indexing_with_unknown_type() -> Result<(), TypeCheckError> {
        check("def f (s: *char) { let i = ftell(0); s[i] = 'a'; }")
    }

    #[test]
    fn type_error_for_indexing_unknown_type() {
        expect_err!(
            "let p = calloc(4, 1); p[1] = 'a';",
            TypeCheckErrorKind::NotIndexable(Type::Unknown)
        );
    }

    #[test]
    fn type_error_for_number_stored_in_bool_pointer() {
        expect_mismatch!("def f (flags: *bool) { flags[0] = 300; }");
    }

    #[test]
    fn supports_return_matching_declared_return_type() -> Result<(), TypeCheckError> {
        check("def f (a: u32): u32 { return a; }")
    }

    #[test]
    fn supports_implicit_return_matching_declared_return_type() -> Result<(), TypeCheckError> {
        check("def f (a: i32, b: i32): i32 { a + b }")
    }

    #[test]
    fn supports_literal_return_coercing_to_declared_return_type() -> Result<(), TypeCheckError> {
        check("def one (): u64 { 1 }")
    }

    #[test]
    fn supports_multiple_returns_matching_declared_return_type() -> Result<(), TypeCheckError> {
        check("def f (a: u32, b: u32): u32 { return a; return b; }")
    }

    #[test]
    fn type_error_for_return_not_matching_declared_return_type() {
        expect_mismatch!("def f (a: i32): u32 { return a; }");
    }

    #[test]
    fn type_error_for_implicit_return_not_matching_declared_return_type() {
        expect_mismatch!("def f (a: u32): bool { a }");
    }

    #[test]
    fn type_error_for_number_literal_returned_from_bool_function() {
        expect_mismatch!("def f (): bool { 1 }");
    }

    #[test]
    fn type_error_for_nested_return_not_matching_declared_return_type() {
        expect_mismatch!("def f (a: i32): u32 { if true { return a; } return 1; }");
    }

    #[test]
    fn type_error_for_empty_body_with_declared_return_type() {
        assert!(check("def f (): u32 { }").is_err());
    }

    #[test]
    fn type_error_for_missing_return_with_declared_return_type() {
        assert!(check("def f (a: u32): u32 { let b = a + 1; }").is_err());
    }

    #[test]
    fn type_error_for_empty_return_with_declared_return_type() {
        assert!(check("def f (): bool { return; }").is_err());
    }

    #[test]
    fn supports_missing_return_without_declared_return_type() -> Result<(), TypeCheckError> {
        check("def f (a: u32) { let b = a + 1; }")
    }

    #[test]
    fn type_error_for_value_returned_from_function_without_return_type() {
        expect_err!(
            "def f (a: u32) { return a; }",
            TypeCheckErrorKind::TypeMismatch { ref expected, .. } if **expected == Type::Unit
        );
    }

    #[test]
    fn type_error_for_implicit_return_from_function_without_return_type() {
        expect_err!(
            "def f (a: u32) { a }",
            TypeCheckErrorKind::TypeMismatch { ref expected, .. } if **expected == Type::Unit
        );
    }

    #[test]
    fn supports_empty_return_from_function_without_return_type() -> Result<(), TypeCheckError> {
        check("def f (a: u32) { if a > 1 { return; } }")
    }

    #[test]
    fn type_error_for_unit_function_result_assigned_to_number() {
        expect_err!(
            "def f () { } let a: u32 = f();",
            TypeCheckErrorKind::TypeMismatch { ref got, .. } if **got == Type::Unit
        );
    }

    #[test]
    fn supports_correct_number_of_args() -> Result<(), TypeCheckError> {
        check("def add (a: u32, b: u32): u32 {a + b} add(1, 2);")
    }

    #[test]
    fn supports_calling_zero_arg_function_with_no_args() -> Result<(), TypeCheckError> {
        check("def f () {} f();")
    }

    #[test]
    fn type_error_for_too_few_args() {
        expect_err!(
            "def add (a: u32, b: u32): u32 {a + b} add(1);",
            TypeCheckErrorKind::IncorrectNumberOfArgs {
                expected: 2,
                given: 1
            }
        );
    }

    #[test]
    fn type_error_for_too_many_args() {
        expect_err!(
            "def add (a: u32, b: u32): u32 {a + b} add(1, 2, 3);",
            TypeCheckErrorKind::IncorrectNumberOfArgs {
                expected: 2,
                given: 3
            }
        );
    }

    #[test]
    fn type_error_for_no_args_to_function_with_params() {
        expect_err!(
            "def f (a: u32) {} f();",
            TypeCheckErrorKind::IncorrectNumberOfArgs {
                expected: 1,
                given: 0
            }
        );
    }

    #[test]
    fn type_error_for_args_to_zero_arg_function() {
        expect_err!(
            "def f () {} f(1);",
            TypeCheckErrorKind::IncorrectNumberOfArgs {
                expected: 0,
                given: 1
            }
        );
    }

    #[test]
    fn type_error_for_incorrect_number_of_args_in_nested_call() {
        expect_err!(
            "def add (a: u32, b: u32): u32 {a + b} let x: u32 = add(add(1), 2);",
            TypeCheckErrorKind::IncorrectNumberOfArgs {
                expected: 2,
                given: 1
            }
        );
    }

    #[test]
    fn type_error_for_incorrect_number_of_args_inside_function_body() {
        expect_err!(
            "def f (a: u32): u32 {a} def g (): u32 { f(1, 2) }",
            TypeCheckErrorKind::IncorrectNumberOfArgs {
                expected: 1,
                given: 2
            }
        );
    }

    #[test]
    fn supports_any_number_of_args_to_extern() -> Result<(), TypeCheckError> {
        check("printf(\"%d\"); printf(\"%d %d\", 1, 2);")
    }

    #[test]
    fn supports_struct_instantiation() -> Result<(), TypeCheckError> {
        check(&format!("{POINT} let p = P {{ x: 1, y: true }};"))
    }

    #[test]
    fn supports_struct_access() -> Result<(), TypeCheckError> {
        check(&format!(
            "{POINT} let p = P {{ x: 1, y: true }}; let a: u32 = p.x; let b: bool = p.y;"
        ))
    }

    #[test]
    fn supports_struct_field_assignment() -> Result<(), TypeCheckError> {
        check(&format!(
            "{POINT} let p = P {{ x: 1, y: true }}; p.x = 5; p.y = false;"
        ))
    }

    #[test]
    fn supports_struct_field_upcasting() -> Result<(), TypeCheckError> {
        check("struct P { x: u64 } let a: u8 = 1; let p = P { x: a };")
    }

    #[test]
    fn supports_struct_access_in_condition() -> Result<(), TypeCheckError> {
        check(&format!(
            "{POINT} let p = P {{ x: 1, y: true }}; if p.y {{ p.x = 2; }}"
        ))
    }

    #[test]
    fn struct_access_has_field_type() {
        let program = checked(&format!(
            "{POINT} let p = P {{ x: 1, y: true }}; let a = p.y;"
        ));
        let StatementOrExpression::Statement(statement) = &program.block.children[2] else {
            panic!("expected statement");
        };
        let Statement::Declaration(declaration) = statement.as_ref() else {
            panic!("expected declaration");
        };
        assert_eq!(declaration.dtype, Some(Type::Boolean));
    }

    #[test]
    fn unknown_generic_argument_error_points_at_argument() {
        let source = "struct N<T> { v: T, } struct B { c: *N<Missing>, }";
        let error = check(source).unwrap_err();
        assert!(matches!(&error.kind, TypeCheckErrorKind::UnknownType(name) if name == "Missing"));
        assert_eq!(error.span.col as usize, source.find("Missing").unwrap());
    }

    #[test]
    fn type_error_for_struct_field_mismatch() {
        expect_err!(
            format!("{POINT} let p = P {{ x: true, y: true }};"),
            TypeCheckErrorKind::TypeMismatch {
                ref expected,
                ref got,
            } if **expected == Type::Int(IntType::U32) && **got == Type::Boolean
        );
    }

    #[test]
    fn type_error_for_unknown_field_in_instantiation() {
        expect_err!(
            format!("{POINT} let p = P {{ x: 1, z: 2 }};"),
            TypeCheckErrorKind::NoSuchField { ref field_name, .. } if field_name == "z"
        );
    }

    #[test]
    fn type_error_for_missing_field_in_instantiation() {
        expect_err!(
            format!("{POINT} let p = P {{ x: 1 }};"),
            TypeCheckErrorKind::MissingField { ref field_name, .. } if field_name == "y"
        );
    }

    #[test]
    fn type_error_for_unknown_field_in_access() {
        expect_err!(
            format!("{POINT} let p = P {{ x: 1, y: true }}; p.z;"),
            TypeCheckErrorKind::NoSuchField { ref field_name, .. } if field_name == "z"
        );
    }

    #[test]
    fn type_error_for_struct_field_assignment_mismatch() {
        expect_mismatch!(format!("{POINT} let p = P {{ x: 1, y: true }}; p.y = 1;"));
    }

    #[test]
    fn type_error_for_instantiating_non_struct() {
        expect_err!(
            "let a = 1; let p = (a { x: 1 });",
            TypeCheckErrorKind::NotAStruct(_)
        );
    }

    #[test]
    fn type_error_for_unknown_struct() {
        expect_err!(
            "let p = Q { x: 1 };",
            TypeCheckErrorKind::IdentifierNotFound(ref name) if name == "Q"
        );
    }

    #[test]
    fn type_error_for_accessing_field_of_non_struct() {
        expect_err!("let a = 1; a.x;", TypeCheckErrorKind::NotAStruct(_));
    }

    #[test]
    fn type_error_for_accessing_field_of_pointer_names_the_type() {
        expect_err!(
            format!("{POINT} let s = P {{ x: 1, y: true }}; let p = &s; p.x;"),
            TypeCheckErrorKind::NotAStruct(ref name) if name == "*P"
        );
    }

    #[test]
    fn supports_pointer_declaration() -> Result<(), TypeCheckError> {
        check("let x: i64 = 1; let p: *i64 = &x;")
    }

    #[test]
    fn supports_assign_through_pointer() -> Result<(), TypeCheckError> {
        check("let x: i64 = 1; let p = &x; *p = 2; *p += 3;")
    }

    #[test]
    fn supports_assign_through_pointer_to_pointer() -> Result<(), TypeCheckError> {
        check("let x: i64 = 1; let p = &x; let pp = &p; **pp = 2;")
    }

    #[test]
    fn supports_upcasting_through_pointer() -> Result<(), TypeCheckError> {
        check("let x: u64 = 1; let p = &x; let small: u8 = 2; *p = small;")
    }

    #[test]
    fn supports_assign_to_field_through_pointer() -> Result<(), TypeCheckError> {
        check(&format!(
            "{POINT} let s = P {{ x: 1, y: true }}; let p = &s; (*p).x = 5; (*p).y = false;"
        ))
    }

    #[test]
    fn supports_increment_through_pointer() -> Result<(), TypeCheckError> {
        check("let x: i64 = 1; let p = &x; (*p)++;")
    }

    #[test]
    fn supports_passing_address_to_function() -> Result<(), TypeCheckError> {
        check("def inc(p: *i64) { *p = *p + 1; } let x: i64 = 1; inc(&x);")
    }

    #[test]
    fn supports_passing_struct_address_to_function() -> Result<(), TypeCheckError> {
        check(&format!(
            "{POINT} def set(p: *P) {{ (*p).x = 2; }} let s = P {{ x: 1, y: true }}; set(&s);"
        ))
    }

    #[test]
    fn supports_returning_pointer_arg() -> Result<(), TypeCheckError> {
        check("def id(p: *i64): *i64 { return p; } let x: i64 = 1; let q: *i64 = id(&x);")
    }

    #[test]
    fn supports_address_of_untyped_variable() -> Result<(), TypeCheckError> {
        check("def f(p: *i64) {} let x = 1; f(&x);")
    }

    #[test]
    fn type_error_for_mismatched_pointer_declaration() {
        expect_mismatch!("let x: u8 = 1; let p: *i64 = &x;");
        expect_mismatch!("let x: i64 = 1; let p: i64 = &x;");
    }

    #[test]
    fn type_error_for_assign_through_pointer_mismatch() {
        expect_mismatch!("let x: bool = true; let p = &x; *p = 2;");
        expect_mismatch!("let x: i64 = 1; let p = &x; *p = true;");
    }

    #[test]
    fn type_error_for_downcasting_through_pointer() {
        expect_mismatch!("let x: u8 = 1; let p = &x; let big: u64 = 2; *p = big;");
    }

    #[test]
    fn type_error_for_field_through_pointer_mismatch() {
        expect_mismatch!(format!(
            "{POINT} let s = P {{ x: 1, y: true }}; let p = &s; (*p).y = 1;"
        ));
    }

    #[test]
    fn type_error_for_passing_wrong_pointer_type() {
        expect_mismatch!("def f(p: *i64) {} let x: u8 = 1; f(&x);");
        expect_mismatch!("def f(p: *i64) {} let x: i64 = 1; f(x);");
        expect_mismatch!("def f(p: i64) {} let x: i64 = 1; f(&x);");
    }

    #[test]
    fn type_error_for_deref_assign_to_non_pointer() {
        expect_err!(
            "let x: i64 = 1; *x = 2;",
            TypeCheckErrorKind::NotDerefable(_)
        );
    }

    #[test]
    fn type_error_for_duplicate_struct_definition() {
        expect_err!(
            format!("{POINT} {POINT}"),
            TypeCheckErrorKind::DuplicateIdentifiers(_)
        );
    }

    #[test]
    fn type_error_for_duplicate_enum_variant() {
        expect_err!(
            "enum Color { Red, Green, Red }",
            TypeCheckErrorKind::DuplicateIdentifiers(_)
        );
    }

    const NODE: &str = "struct N { v: i64, next: *N, children: **N }";

    #[test]
    fn supports_self_referential_struct_pointers() -> Result<(), TypeCheckError> {
        check(&format!(
            "{NODE} let b = N {{ v: 2, next: 0 as *N, children: 0 as **N }};              let a = N {{ v: 1, next: &b, children: 0 as **N }};              let x: i64 = (*a.next).v;              let n: N = *a.next;              let c: *N = a.children[0];              (*a.next).next = &a;"
        ))
    }

    #[test]
    fn type_error_for_self_referential_struct_without_pointer() {
        expect_err!(
            "struct N { v: i64, next: N }",
            TypeCheckErrorKind::RecursiveStruct(ref name) if name == "N"
        );
    }

    #[test]
    fn type_error_for_unknown_field_through_self_referential_pointer() {
        expect_err!(
            format!("{NODE} let a = N {{ v: 1, next: 0 as *N, children: 0 as **N }}; (*a.next).z;"),
            TypeCheckErrorKind::NoSuchField { ref field_name, .. } if field_name == "z"
        );
    }

    #[test]
    fn type_error_for_assigning_wrong_pointer_to_self_referential_field() {
        expect_mismatch!(format!(
            "{NODE} {POINT} let p = P {{ x: 1, y: true }};              let a = N {{ v: 1, next: 0 as *N, children: 0 as **N }}; a.next = &p;"
        ));
    }

    #[test]
    fn supports_comparing_pointer_to_null() -> Result<(), TypeCheckError> {
        check("let p: *u8 = malloc(1); let a = p != null; let b = null == p;")
    }

    #[test]
    fn supports_assigning_null_to_pointer() -> Result<(), TypeCheckError> {
        check("struct N { next: *N, } let n = N { next: null }; let p: *u8 = null; p = null;")
    }

    #[test]
    fn type_error_for_null_as_int() {
        expect_mismatch!("let a: u64 = null;");
        expect_mismatch!("let a: u64 = 1; let b = a == null;");
    }

    #[test]
    fn type_error_for_comparing_pointer_to_number() {
        expect_mismatch!("let p: *u8 = malloc(1); let a = p != 0;");
    }
}
