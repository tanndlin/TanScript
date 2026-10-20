use crate::{type_check::Type, types::IntType};

pub fn unify(a: &Type, b: &Type) -> Option<Type> {
    match (a, b) {
        _ if a == b => Some(a.clone()),
        (Type::Int(from), Type::Int(to)) if from.widens_to(*to) => Some(b.clone()),
        (Type::Int(_), Type::Int(IntType::Coercible) | Type::Boolean)
        | (Type::Pointer(_), Type::Null) => Some(a.clone()),
        (Type::Char, Type::Int(IntType::Coercible)) => Some(Type::Char),
        (Type::Function(f), _) if f.return_type == *b => Some(b.clone()),
        (Type::Unknown, _) => Some(b.clone()),
        _ => None,
    }
}

#[cfg(test)]
mod test {
    use crate::{
        ast::StatementOrExpression,
        lex::Lexer,
        parse::parse,
        type_check::{Type, TypeCheckError, TypeCheckErrorKind, TypeScope, check_types_expression},
        types::IntType,
    };

    fn infer_last(code: &str) -> Result<Type, TypeCheckError> {
        let mut program = parse(Lexer::new(code).unwrap()).unwrap();
        let mut type_scope = TypeScope::default();
        let (last, rest) = program.block.children.split_last_mut().unwrap();
        for child in rest {
            child.check_types(&mut type_scope)?;
        }
        let StatementOrExpression::Expression(expression) = last else {
            panic!("expected an expression");
        };
        check_types_expression(expression, &mut type_scope)
    }

    #[test]
    fn infers_type_from_literal() {
        assert_eq!(infer_last("1;").unwrap(), Type::Int(IntType::Coercible));
    }

    #[test]
    fn infers_type_from_variable() {
        assert_eq!(
            infer_last("let a: u32 = 1; a;").unwrap(),
            Type::Int(IntType::U32)
        );
    }

    #[test]
    fn infers_type_from_binary_op_literals() {
        assert_eq!(infer_last("1 + 2;").unwrap(), Type::Int(IntType::Coercible));
    }

    #[test]
    fn infers_type_from_untyped_variable() {
        assert_eq!(
            infer_last("let a = 1; a;").unwrap(),
            Type::Int(IntType::Coercible)
        );
    }

    #[test]
    fn infers_type_from_typed_variable_plus_literal() {
        assert_eq!(
            infer_last("let a: i32 = 1; a + 2;").unwrap(),
            Type::Int(IntType::I32)
        );
    }

    #[test]
    fn infers_type_from_literal_plus_typed_variable() {
        assert_eq!(
            infer_last("let a: u64 = 1; 2 + a;").unwrap(),
            Type::Int(IntType::U64)
        );
    }

    #[test]
    fn infers_type_from_same_typed_variables() {
        assert_eq!(
            infer_last("let a: i64 = 1; let b: i64 = 2; a + b;").unwrap(),
            Type::Int(IntType::I64)
        );
    }

    #[test]
    fn infers_type_from_nested_binary_ops() {
        assert_eq!(
            infer_last("let a: u32 = 1; let b: u32 = 2; a + b * 3;").unwrap(),
            Type::Int(IntType::U32)
        );
    }

    #[test]
    fn infers_upcast_from_u8_plus_u32() {
        assert_eq!(
            infer_last("let a: u8 = 1; let b: u32 = 2; a + b;").unwrap(),
            Type::Int(IntType::U32)
        );
    }

    #[test]
    fn infers_upcast_from_u32_plus_u8() {
        assert_eq!(
            infer_last("let a: u32 = 1; let b: u8 = 2; a + b;").unwrap(),
            Type::Int(IntType::U32)
        );
    }

    #[test]
    fn infers_upcast_from_u8_plus_u64() {
        assert_eq!(
            infer_last("let a: u8 = 1; let b: u64 = 2; a * b;").unwrap(),
            Type::Int(IntType::U64)
        );
    }

    #[test]
    fn infers_upcast_from_u64_plus_u32() {
        assert_eq!(
            infer_last("let a: u64 = 1; let b: u32 = 2; a - b;").unwrap(),
            Type::Int(IntType::U64)
        );
    }

    #[test]
    fn infers_upcast_from_i32_plus_i64() {
        assert_eq!(
            infer_last("let a: i32 = 1; let b: i64 = 2; a + b;").unwrap(),
            Type::Int(IntType::I64)
        );
    }

    #[test]
    fn infers_upcast_from_u32_plus_i64() {
        assert_eq!(
            infer_last("let a: u32 = 1; let b: i64 = 2; a + b;").unwrap(),
            Type::Int(IntType::I64)
        );
    }

    #[test]
    fn infers_upcast_from_i64_plus_u8() {
        assert_eq!(
            infer_last("let a: i64 = 1; let b: u8 = 2; a + b;").unwrap(),
            Type::Int(IntType::I64)
        );
    }

    #[test]
    fn type_error_for_mismatched_variables() {
        let result = infer_last("let a: u32 = 1; let b: i32 = 2; a + b;");
        assert!(
            matches!(
                result,
                Err(TypeCheckError {
                    kind: TypeCheckErrorKind::TypeMismatch { .. },
                    ..
                })
            ),
            "expected TypeMismatch, got {result:?}"
        );
    }

    #[test]
    fn type_error_for_mismatched_nested_binary_ops() {
        let result = infer_last("let a: u64 = 1; let b: i32 = 2; (a + 1) * (b + 2);");
        assert!(
            matches!(
                result,
                Err(TypeCheckError {
                    kind: TypeCheckErrorKind::TypeMismatch { .. },
                    ..
                })
            ),
            "expected TypeMismatch, got {result:?}"
        );
    }

    #[test]
    fn type_error_for_struct_in_binary_op() {
        for code in [
            "struct P { a: i64, } let p = P { a: 1 }; p % p;",
            "struct P { a: i64, } let p = P { a: 1 }; p + printf(\"x\");",
            "struct P { a: i64, } let p = P { a: 1 }; p == p;",
        ] {
            let result = infer_last(code);
            assert!(
                matches!(
                    result,
                    Err(TypeCheckError {
                        kind: TypeCheckErrorKind::TypeMismatch { .. },
                        ..
                    })
                ),
                "expected TypeMismatch for {code}, got {result:?}"
            );
        }
    }

    #[test]
    fn type_error_for_struct_as_function_arg() {
        for code in [
            "struct P { a: i64, } let p = P { a: 1 }; printf(\"%d\", p);",
            "struct P { a: i64, } let p = P { a: 1 }; f(1, 2, 3, 4, p);",
        ] {
            let result = infer_last(code);
            assert!(
                matches!(
                    result,
                    Err(TypeCheckError {
                        kind: TypeCheckErrorKind::StructAsArgument,
                        ..
                    })
                ),
                "expected StructAsArgument for {code}, got {result:?}"
            );
        }
    }

    #[test]
    fn struct_as_defined_function_arg() {
        let result = infer_last(
            "struct P { a: i64, } def f(p: P): i64 { return p.a; } let p = P { a: 1 }; f(p);",
        );
        assert!(result.is_ok(), "{result:?}");

        let result = infer_last(
            "struct P { a: i64, } def f(x: i64): i64 { return x; } let p = P { a: 1 }; f(p);",
        );
        assert!(
            matches!(
                result,
                Err(TypeCheckError {
                    kind: TypeCheckErrorKind::TypeMismatch { .. },
                    ..
                })
            ),
            "{result:?}"
        );
    }

    #[test]
    fn resolves_struct_type_names() {
        for code in [
            "struct P { a: i64, } def f(p: P): i64 { return 1; } f;",
            "struct P { a: i64, } def f(p: *P): *P { return p; } f;",
            "struct P { a: i64, } struct Q { p: P, ptr: *P, } let q: *Q = 0 as *Q; q;",
        ] {
            let result = infer_last(code);
            assert!(result.is_ok(), "expected Ok for {code}, got {result:?}");
        }
    }

    #[test]
    fn resolved_struct_type_is_the_definition() {
        let Type::Pointer(inner) = infer_last("struct P { a: i64, } 0 as *P;").unwrap() else {
            panic!("expected a pointer");
        };
        assert!(matches!(*inner, Type::Struct(def) if def.name == "P"));
    }

    #[test]
    fn type_error_for_unknown_type() {
        for code in [
            "def f(p: P): i64 { return 1; } f;",
            "def f(): *P { return 0; } f;",
            "struct Q { p: P, } 1;",
            "let x = 1; let y: x = 1; y;",
            "0 as *P;",
        ] {
            let result = infer_last(code);
            assert!(
                matches!(
                    &result,
                    Err(TypeCheckError {
                        kind: TypeCheckErrorKind::UnknownType(_),
                        ..
                    })
                ),
                "expected UnknownType for {code}, got {result:?}"
            );
        }
    }

    fn pointer_to(t: Type) -> Type {
        Type::Pointer(Box::new(t))
    }

    #[test]
    fn infers_address_of_type() {
        assert_eq!(
            infer_last("let x: i64 = 1; &x;").unwrap(),
            pointer_to(Type::Int(IntType::I64))
        );
        assert_eq!(
            infer_last("let c: char = 'a'; &c;").unwrap(),
            pointer_to(Type::Char)
        );
        assert_eq!(
            infer_last("let x: u8 = 1; let p = &x; &p;").unwrap(),
            pointer_to(pointer_to(Type::Int(IntType::U8)))
        );
    }

    #[test]
    fn infers_address_of_index_type() {
        assert_eq!(
            infer_last("let s: *char = malloc(4); &s[1];").unwrap(),
            pointer_to(Type::Char)
        );
    }

    #[test]
    fn infers_address_of_struct_type() {
        let Type::Pointer(inner) =
            infer_last("struct P { a: u8, } let s = P { a: 1 }; &s;").unwrap()
        else {
            panic!("expected a pointer");
        };
        assert!(matches!(*inner, Type::Struct(def) if def.name == "P"));
    }

    #[test]
    fn infers_address_of_field_type() {
        assert_eq!(
            infer_last("struct P { a: u8, } let s = P { a: 1 }; &s.a;").unwrap(),
            pointer_to(Type::Int(IntType::U8))
        );
    }

    #[test]
    fn infers_deref_type() {
        assert_eq!(
            infer_last("let x: u8 = 1; let p = &x; *p;").unwrap(),
            Type::Int(IntType::U8)
        );
        assert_eq!(
            infer_last("let s: *char = malloc(4); *s;").unwrap(),
            Type::Char
        );
        assert_eq!(
            infer_last("let x: i64 = 1; let p = &x; let pp = &p; **pp;").unwrap(),
            Type::Int(IntType::I64)
        );
        assert_eq!(
            infer_last("let x: u32 = 1; *&x;").unwrap(),
            Type::Int(IntType::U32)
        );
    }

    #[test]
    fn infers_field_type_through_deref() {
        assert_eq!(
            infer_last(
                "struct P { a: u8, b: bool, } let s = P { a: 1, b: true }; let p = &s; (*p).b;"
            )
            .unwrap(),
            Type::Boolean
        );
    }

    #[test]
    fn infers_deref_of_pointer_arg() {
        assert_eq!(
            infer_last("def f(p: *u32): u32 { return *p; } let x: u32 = 1; f(&x);").unwrap(),
            Type::Int(IntType::U32)
        );
    }

    #[test]
    fn struct_pointer_is_not_a_struct_argument() {
        assert_eq!(
            infer_last(
                "struct P { a: u8, } def f(p: *P): u8 { return (*p).a; } let s = P { a: 1 }; f(&s);"
            )
            .unwrap(),
            Type::Int(IntType::U8)
        );
    }

    #[test]
    fn type_error_for_deref_of_non_pointer() {
        for code in [
            "let x: i64 = 1; *x;",
            "let b = true; *b;",
            "struct P { a: u8, } let s = P { a: 1 }; *s;",
            "let x: i64 = 1; let p = &x; **p;",
        ] {
            let result = infer_last(code);
            assert!(
                matches!(
                    result,
                    Err(TypeCheckError {
                        kind: TypeCheckErrorKind::NotDerefable(_),
                        ..
                    })
                ),
                "expected NotDerefable for {code}, got {result:?}"
            );
        }
    }

    #[test]
    fn type_error_for_field_access_through_non_struct_pointer() {
        let result = infer_last("let x: i64 = 1; let p = &x; (*p).a;");
        assert!(result.is_err(), "expected an error, got {result:?}");
    }

    #[test]
    fn infers_type_from_function_call() {
        assert_eq!(
            infer_last("def add (a: u32, b: u32): u32 {a + b} add(1, 2);").unwrap(),
            Type::Int(IntType::U32)
        );
    }

    #[test]
    fn infers_type_from_function_call_plus_variable() {
        assert_eq!(
            infer_last("def add (a: u32, b: u32): u32 {a + b} let c: u32 = 3; add(1, 2) + c;")
                .unwrap(),
            Type::Int(IntType::U32)
        );
    }

    #[test]
    fn infers_type_from_unary_op() {
        assert_eq!(
            infer_last("let a: i32 = 1; -a;").unwrap(),
            Type::Int(IntType::I32)
        );
    }

    #[test]
    fn infers_type_from_postfix_op() {
        assert_eq!(
            infer_last("let a: i32 = 1; a++;").unwrap(),
            Type::Int(IntType::I32)
        );
    }

    #[test]
    fn infers_type_from_boolean_literal() {
        assert_eq!(infer_last("true;").unwrap(), Type::Boolean);
        assert_eq!(infer_last("false;").unwrap(), Type::Boolean);
    }

    #[test]
    fn infers_type_from_boolean_variable() {
        assert_eq!(infer_last("let a = true; a;").unwrap(), Type::Boolean);
    }

    #[test]
    fn infers_type_from_boolean_binary_op() {
        assert_eq!(infer_last("true && false;").unwrap(), Type::Boolean);
    }

    // Might want to enable this again once I implement `true as u32`
    // #[test]
    // fn type_error_for_boolean_plus_number() {
    //     let result = infer_last("let a: i32 = 1; true + a;");
    //     assert!(
    //         matches!(
    //             result,
    //             Err(TypeCheckError {
    //                 kind: TypeCheckErrorKind::TypeMismatch { .. },
    //                 ..
    //             })
    //         ),
    //         "expected TypeMismatch, got {result:?}"
    //     );
    // }

    #[test]
    fn infers_boolean_from_comparison() {
        for op in ["<", "<=", ">", ">=", "==", "!="] {
            assert_eq!(
                infer_last(&format!("let a: i32 = 1; let b: i32 = 2; a {op} b;")).unwrap(),
                Type::Boolean,
                "{op}"
            );
        }
    }

    #[test]
    fn infers_boolean_from_not() {
        assert_eq!(infer_last("let a: i32 = 1; !a;").unwrap(), Type::Boolean);
    }

    #[test]
    fn type_error_for_comparing_mismatched_types() {
        let result = infer_last("let a: u32 = 1; let b: i32 = 2; a < b;");
        assert!(
            matches!(
                result,
                Err(TypeCheckError {
                    kind: TypeCheckErrorKind::TypeMismatch { .. },
                    ..
                })
            ),
            "expected TypeMismatch, got {result:?}"
        );
    }
}
